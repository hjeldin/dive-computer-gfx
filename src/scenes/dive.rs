use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::Rgb565,
    prelude::{Point, RgbColor, Size, WebColors},
    primitives::PrimitiveStyleBuilder,
};
use profont::{PROFONT_18_POINT, PROFONT_7_POINT, PROFONT_9_POINT};

use crate::{
    common::{Drawable, State},
    components::{compass::Compass, depth_profile::DepthProfile, label::Label},
    scene,
    utils::background::Background,
};
use core::format_args;
use core::option::Option::Some;
use core::option::Option::None;
use libm::ceil;

/// Width of one cell in the data table.
const CELL_WIDTH: i32 = 99;
/// Height of one row in the data table: a title bar and a value.
const ROW_HEIGHT: i32 = 37;
/// Top of the content area, below the status bar.
const CONTENT_TOP: i32 = 16;
const TITLE_BAR_HEIGHT: u32 = 13;

/// Data table cells as (title, update), laid out in two columns, left to right then down.
/// NDT, ceiling and ascent rate have no data source yet: they show
/// placeholder values derived from pressure until the deco model is wired in.
const CELLS: [(&str, fn(&mut Label, &State)); 6] = [
    ("Depth", show_metres),
    ("NDT", show_minutes),
    ("Max Depth", show_metres),
    ("Runtime", show_runtime),
    ("Ceiling", show_minutes),
    ("Asc Rate", show_minutes),
];

pub fn build_dive() -> scene::Scene {
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

    for (index, (title, show)) in CELLS.iter().enumerate() {
        let x = (index as i32 % 2) * CELL_WIDTH;
        let top = CONTENT_TOP + 2 + (index as i32 / 2) * ROW_HEIGHT;

        // Labels centre their text on their box, with the baseline at the box's middle
        scene.add_component(Drawable::Background(Background::new(
            Point::new(x, top),
            Size::new(CELL_WIDTH as u32 - 1, TITLE_BAR_HEIGHT),
            Some(title_bar_style),
        )));
        scene.add_component(Drawable::Label(Label::new(
            Point::new(x, top),
            Size::new(CELL_WIDTH as u32, 20),
            title,
            title_style,
        )));
        let mut value = Label::new(
            Point::new(x, top + 21),
            Size::new(CELL_WIDTH as u32, 24),
            "",
            value_style,
        )
        .with_background(Rgb565::BLACK);
        value.set_on_update(*show);
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
    let profile_top = table_bottom + 6;
    scene.add_component(Drawable::DepthProfile(DepthProfile::new(
        Point::new(2, profile_top),
        Size::new(320 - 8, (240 - 2 - profile_top) as u32),
        MonoTextStyle::new(&PROFONT_7_POINT, Rgb565::WHITE),
    )));

    scene
}

fn show_metres(label: &mut Label, state: &State) {
    label.set_label_fmt(format_args!("{:.1}m", state.depth_m()));
}

fn show_runtime(label: &mut Label, state: &State) {
    let seconds = state.dive_time_ms / 1_000;
    label.set_label_fmt(format_args!("{}:{:02}", seconds / 60, seconds % 60));
}

fn show_minutes(label: &mut Label, state: &State) {
    label.set_label_fmt(format_args!("{}min", ceil(state.pressure as f64) as u32));
}
