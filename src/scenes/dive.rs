use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::Rgb565,
    prelude::{Point, RgbColor, Size, WebColors},
    primitives::PrimitiveStyleBuilder,
};
use profont::{PROFONT_18_POINT, PROFONT_7_POINT, PROFONT_9_POINT};

use crate::{
    common::{DecoDisplay, Drawable, State},
    components::{compass::Compass, depth_profile::DepthProfile, label::Label},
    scene,
    utils::background::Background,
};
use core::format_args;
use core::option::Option::Some;
use core::option::Option::None;
use libm::roundf;

/// Width of one cell in the data table.
const CELL_WIDTH: i32 = 99;
/// Height of one row in the data table: a title bar and a value.
const ROW_HEIGHT: i32 = 37;
/// Top of the content area, below the status bar.
const CONTENT_TOP: i32 = 16;
const TITLE_BAR_HEIGHT: u32 = 13;

/// One cell of the data table: a title bar and a value below it.
struct Cell {
    title: &'static str,
    /// Changes the title with the state, for cells whose meaning changes.
    title_update: Option<fn(&mut Label, &State)>,
    value: fn(&mut Label, &State),
}

/// Data table cells, laid out in two columns, left to right then down.
/// With the `raw-pressure` feature the TTS cell shows the raw sensor pressure
/// instead, for testing the pressure sensor.
const CELLS: [Cell; 6] = [
    Cell { title: "Depth", title_update: None, value: show_depth },
    Cell { title: "NDL", title_update: Some(show_deco_title), value: show_deco },
    Cell { title: "Max Depth", title_update: None, value: show_max_depth },
    Cell { title: "Runtime", title_update: None, value: show_runtime },
    #[cfg(not(feature = "raw-pressure"))]
    Cell { title: "TTS", title_update: None, value: show_tts },
    #[cfg(feature = "raw-pressure")]
    Cell { title: "mbar", title_update: None, value: show_pressure },
    Cell { title: "Asc Rate", title_update: None, value: show_ascent_rate },
];

/// Top of the depth profile, below the data table.
const PROFILE_TOP: i32 = CONTENT_TOP + 3 * ROW_HEIGHT + 6;

/// The dive scene's depth profile, empty, at its place in the scene.
/// Put it in static memory and pass it to `build_dive`: see `Drawable`.
pub fn new_depth_profile() -> DepthProfile {
    DepthProfile::new(
        Point::new(2, PROFILE_TOP),
        Size::new(320 - 8, (240 - 2 - PROFILE_TOP) as u32),
        MonoTextStyle::new(&PROFONT_7_POINT, Rgb565::WHITE),
    )
}

/// Builds the dive scene around `profile`, made by `new_depth_profile`.
pub fn build_dive(profile: &'static mut DepthProfile) -> scene::Scene {
    let mut scene = scene::Scene::new();
    scene.add_component(Drawable::Background(Background::new(
        Point::new(0, CONTENT_TOP),
        Size::new(320, 240 - CONTENT_TOP as u32),
        None,
    )));

    let title_bar_style = PrimitiveStyleBuilder::new()
        .fill_color(Rgb565::CSS_DIM_GRAY)
        .build();
    let title_style = MonoTextStyle::new(&PROFONT_9_POINT, Rgb565::WHITE);
    let value_style = MonoTextStyle::new(&PROFONT_18_POINT, Rgb565::GREEN);

    for (index, cell) in CELLS.iter().enumerate() {
        let x = (index as i32 % 2) * CELL_WIDTH;
        let top = CONTENT_TOP + 2 + (index as i32 / 2) * ROW_HEIGHT;

        // Labels centre their text on their box, with the baseline at the box's middle
        scene.add_component(Drawable::Background(Background::new(
            Point::new(x, top),
            Size::new(CELL_WIDTH as u32 - 1, TITLE_BAR_HEIGHT),
            Some(title_bar_style),
        )));
        // A background colour lets a changed title erase the old one
        let mut title = Label::new(
            Point::new(x, top),
            Size::new(CELL_WIDTH as u32, 20),
            cell.title,
            title_style,
        )
        .with_background(Rgb565::CSS_DIM_GRAY);
        if let Some(update) = cell.title_update {
            title.set_on_update(update);
        }
        scene.add_component(Drawable::Label(title));
        let mut value = Label::new(
            Point::new(x, top + 21),
            Size::new(CELL_WIDTH as u32, 24),
            "",
            value_style,
        )
        .with_background(Rgb565::BLACK);
        value.set_on_update(cell.value);
        scene.add_component(Drawable::Label(value));
    }

    // Compass beside the table, the same height as it
    let table_bottom = CONTENT_TOP + 3 * ROW_HEIGHT;
    scene.add_component(Drawable::Compass(Compass::new(
        Point::new(2 * CELL_WIDTH + 2, CONTENT_TOP),
        Size::new(320 - 2 * CELL_WIDTH as u32 - 2, (table_bottom - CONTENT_TOP) as u32),
        MonoTextStyle::new(&PROFONT_9_POINT, Rgb565::WHITE),
    )));

    // Depth profile across the full width below
    scene.add_component(Drawable::DepthProfile(profile));

    scene
}

fn show_depth(label: &mut Label, state: &State) {
    label.set_label_fmt(format_args!("{:.1}m", state.depth_m));
}

fn show_max_depth(label: &mut Label, state: &State) {
    label.set_label_fmt(format_args!("{:.1}m", state.max_depth_m));
}

fn show_deco_title(label: &mut Label, state: &State) {
    match state.deco {
        DecoDisplay::Stop { .. } => label.set_label_text("Stop"),
        _ => label.set_label_text("NDL"),
    }
}

fn show_deco(label: &mut Label, state: &State) {
    match state.deco {
        DecoDisplay::Unknown => label.set_label_text("---"),
        DecoDisplay::Ndl(minutes) => label.set_label_fmt(format_args!("{}min", minutes)),
        DecoDisplay::NdlOver99 => label.set_label_text(">99"),
        DecoDisplay::Stop { depth_m, minutes } => {
            label.set_label_fmt(format_args!("{}m {}min", depth_m, minutes))
        }
    }
}

fn show_tts(label: &mut Label, state: &State) {
    match state.tts_min {
        Some(minutes) => label.set_label_fmt(format_args!("{}min", minutes)),
        None => label.set_label_text("---"),
    }
}

fn show_ascent_rate(label: &mut Label, state: &State) {
    // Whole metres; a rounded -0.0 converts to 0
    let rate = roundf(state.ascent_rate_m_min) as i32;
    label.set_label_fmt(format_args!("{}m/min", rate));
}

/// Raw sensor pressure; seven characters up to 9999.9 mbar fit the cell.
#[cfg(feature = "raw-pressure")]
fn show_pressure(label: &mut Label, state: &State) {
    label.set_label_fmt(format_args!("{:.1}", state.pressure));
}

fn show_runtime(label: &mut Label, state: &State) {
    let seconds = state.dive_time_ms / 1_000;
    label.set_label_fmt(format_args!("{}:{:02}", seconds / 60, seconds % 60));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn label() -> Label {
        Label::new(Point::new(0, 0), Size::new(99, 24), "", MonoTextStyle::new(&PROFONT_18_POINT, Rgb565::GREEN))
    }

    fn assert_shown(update: fn(&mut Label, &State), state: &State, expected: &str) {
        let mut label = label();
        update(&mut label, state);
        assert_eq!(label.text(), expected);
    }

    #[test]
    fn depth_and_max_depth() {
        let mut state = State::new();
        state.depth_m = 12.44;
        state.max_depth_m = 18.24;
        assert_shown(show_depth, &state, "12.4m");
        assert_shown(show_max_depth, &state, "18.2m");
    }

    #[test]
    fn deco_cell_and_title() {
        let mut state = State::new();
        let cases = [
            (DecoDisplay::Unknown, "NDL", "---"),
            (DecoDisplay::Ndl(34), "NDL", "34min"),
            (DecoDisplay::NdlOver99, "NDL", ">99"),
            (DecoDisplay::Stop { depth_m: 6, minutes: 3 }, "Stop", "6m 3min"),
        ];
        for (deco, title, value) in cases {
            state.deco = deco;
            assert_shown(show_deco_title, &state, title);
            assert_shown(show_deco, &state, value);
        }
    }

    #[test]
    fn tts_and_ascent_rate() {
        let mut state = State::new();
        assert_shown(show_tts, &state, "---");
        state.tts_min = Some(12);
        assert_shown(show_tts, &state, "12min");
        state.ascent_rate_m_min = 9.4;
        assert_shown(show_ascent_rate, &state, "9m/min");
        state.ascent_rate_m_min = -0.3;
        assert_shown(show_ascent_rate, &state, "0m/min");
        state.ascent_rate_m_min = -18.0;
        assert_shown(show_ascent_rate, &state, "-18m/min");
    }

    #[cfg(feature = "raw-pressure")]
    #[test]
    fn raw_pressure() {
        let mut state = State::new();
        state.pressure = 1013.27;
        assert_shown(show_pressure, &state, "1013.3");
    }

    #[test]
    fn runtime() {
        let mut state = State::new();
        state.dive_time_ms = (23 * 60 + 41) * 1000;
        assert_shown(show_runtime, &state, "23:41");
    }

    #[cfg(not(feature = "raw-pressure"))]
    #[test]
    fn table_titles() {
        let titles: [&str; 6] = CELLS.map(|cell| cell.title);
        assert_eq!(titles, ["Depth", "NDL", "Max Depth", "Runtime", "TTS", "Asc Rate"]);
    }
}
