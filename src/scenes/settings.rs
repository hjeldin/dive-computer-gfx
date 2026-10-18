use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::Rgb565,
    prelude::{Point, RgbColor, Size},
};
use profont::{PROFONT_12_POINT, PROFONT_9_POINT};

use crate::{
    common::{Action, Drawable, Setting, State, UsbMode},
    components::{button::Button, dropdown::Dropdown, label::Label, value_field::ValueField},
    scene,
    state_machine::states::STARTUP_SCENE,
    utils::background::Background,
};
use core::format_args;
use core::option::Option::None;

/// Top of the content area, below the status bar.
const CONTENT_TOP: i32 = 16;
/// Top of the first settings row, below the title.
const ROWS_TOP: i32 = CONTENT_TOP + 26;
/// Distance between the tops of two rows: nine rows fill the screen.
const ROW_HEIGHT: i32 = 22;
const FIELD_HEIGHT: u32 = 20;
/// Width of the column with the setting names.
const NAME_WIDTH: u32 = 120;
/// Left edge of the column with the fields.
const FIELD_LEFT: i32 = 130;
/// Width of a two-digit field, such as the day or the hour.
const SHORT_WIDTH: u32 = 36;
const SEPARATOR_WIDTH: u32 = 16;
const WIDE_WIDTH: u32 = 100;
/// Moves a `Label` down to line its text up with the field text: a label puts
/// the baseline at the middle of its box, a field centres the text on it.
const LABEL_SHIFT: i32 = 4;

/// One field in a settings row: its offset from `FIELD_LEFT`, width, setting
/// and the callback that shows its value.
struct Field {
    offset: i32,
    width: u32,
    setting: Setting,
    show: fn(&mut ValueField, &State),
}

/// A row of value fields with a name on the left and `separator` between the fields.
struct Row {
    name: &'static str,
    separator: &'static str,
    fields: &'static [Field],
}

/// Offset of the second field in a row of short fields.
const SECOND: i32 = (SHORT_WIDTH + SEPARATOR_WIDTH) as i32;
/// Offset of the third field in a row of short fields.
const THIRD: i32 = 2 * SECOND;

/// The value field rows, top to bottom. Selection goes through them in this order.
const ROWS: [Row; 7] = [
    Row {
        name: "Date",
        separator: "/",
        fields: &[
            Field { offset: 0, width: SHORT_WIDTH, setting: Setting::Day, show: show_day },
            Field { offset: SECOND, width: SHORT_WIDTH, setting: Setting::Month, show: show_month },
            Field { offset: THIRD, width: 56, setting: Setting::Year, show: show_year },
        ],
    },
    Row {
        name: "Time",
        separator: ":",
        fields: &[
            Field { offset: 0, width: SHORT_WIDTH, setting: Setting::Hour, show: show_hour },
            Field { offset: SECOND, width: SHORT_WIDTH, setting: Setting::Minute, show: show_minute },
        ],
    },
    Row {
        name: "GF High",
        separator: "",
        fields: &[Field { offset: 0, width: WIDE_WIDTH, setting: Setting::GfHigh, show: show_gf_high }],
    },
    Row {
        name: "GF Low",
        separator: "",
        fields: &[Field { offset: 0, width: WIDE_WIDTH, setting: Setting::GfLow, show: show_gf_low }],
    },
    Row {
        name: "SAC rate",
        separator: "",
        fields: &[Field { offset: 0, width: WIDE_WIDTH, setting: Setting::SacRate, show: show_sac }],
    },
    Row {
        name: "Backlight",
        separator: "",
        fields: &[Field { offset: 0, width: WIDE_WIDTH, setting: Setting::Backlight, show: show_backlight }],
    },
    Row {
        name: "Screen off",
        separator: "",
        fields: &[Field { offset: 0, width: WIDE_WIDTH, setting: Setting::ScreenOff, show: show_screen_off }],
    },
];

/// Builds the settings scene: the date and time, the gradient factors, the
/// estimated SAC rate, the backlight, the idle time before the screen turns
/// off and the USB mode, with a button back to the startup scene.
///
/// Select a field and click it to edit it: next then steps the value up,
/// wrapping at its highest value, and a second click ends editing.
pub fn build_settings() -> scene::Scene {
    let mut scene = scene::Scene::new();
    scene.add_component(Drawable::Background(Background::new(
        Point::new(0, CONTENT_TOP),
        Size::new(320, 240 - CONTENT_TOP as u32),
        None,
    )));

    let title_style = MonoTextStyle::new(&PROFONT_12_POINT, Rgb565::WHITE);
    let name_style = MonoTextStyle::new(&PROFONT_9_POINT, Rgb565::WHITE);
    scene.add_component(Drawable::Label(Label::new(
        Point::new(0, CONTENT_TOP + 2),
        Size::new(320, 22),
        "Settings",
        title_style,
    )));

    let row_top = |row: usize| ROWS_TOP + row as i32 * ROW_HEIGHT;
    let field_size = |width: u32| Size::new(width, FIELD_HEIGHT);

    for (index, row) in ROWS.iter().enumerate() {
        let top = row_top(index);
        add_name(&mut scene, top, row.name, name_style);
        for (position, field) in row.fields.iter().enumerate() {
            if position > 0 {
                // Labels need a background colour, as the dropdown list can cover them
                let separator = Label::new(
                    Point::new(FIELD_LEFT + field.offset - SEPARATOR_WIDTH as i32, top + LABEL_SHIFT),
                    Size::new(SEPARATOR_WIDTH, FIELD_HEIGHT),
                    row.separator,
                    name_style,
                )
                .with_background(Rgb565::BLACK);
                scene.add_component(Drawable::Label(separator));
            }
            let mut value = ValueField::new(
                Point::new(FIELD_LEFT + field.offset, top),
                field_size(field.width),
                field.setting,
            );
            value.set_on_update(field.show);
            scene.add_component(Drawable::ValueField(value));
        }
    }

    let usb_top = row_top(ROWS.len());
    add_name(&mut scene, usb_top, "USB mode", name_style);
    scene.add_component(Drawable::Dropdown(Dropdown::new(
        Point::new(FIELD_LEFT, usb_top),
        field_size(WIDE_WIDTH),
        Setting::UsbMode,
        &UsbMode::LABELS,
        |state| state.settings.usb_mode.index(),
    )));

    // The dropdown list opens over this button
    let mut back = Button::new(
        Point::new(FIELD_LEFT, row_top(ROWS.len() + 1)),
        field_size(WIDE_WIDTH),
        "Back",
    );
    back.set_on_click(|_button: &mut Button| Action::SwitchScene(STARTUP_SCENE));
    scene.add_component(Drawable::Button(back));

    scene.select_next_toggleable();
    scene
}

/// Adds the name of a settings row on the left of the row at `top`.
fn add_name(scene: &mut scene::Scene, top: i32, name: &str, style: MonoTextStyle<'static, Rgb565>) {
    let label = Label::new(Point::new(0, top + LABEL_SHIFT), Size::new(NAME_WIDTH, FIELD_HEIGHT), name, style)
        .with_background(Rgb565::BLACK);
    scene.add_component(Drawable::Label(label));
}

fn show_day(field: &mut ValueField, state: &State) {
    field.set_text_fmt(format_args!("{:02}", state.datetime.date.day));
}

fn show_month(field: &mut ValueField, state: &State) {
    field.set_text_fmt(format_args!("{:02}", state.datetime.date.month));
}

fn show_year(field: &mut ValueField, state: &State) {
    field.set_text_fmt(format_args!("{}", state.datetime.date.year));
}

fn show_hour(field: &mut ValueField, state: &State) {
    field.set_text_fmt(format_args!("{:02}", state.datetime.time.hours));
}

fn show_minute(field: &mut ValueField, state: &State) {
    field.set_text_fmt(format_args!("{:02}", state.datetime.time.minutes));
}

fn show_gf_high(field: &mut ValueField, state: &State) {
    field.set_text_fmt(format_args!("{}%", state.settings.gf_high));
}

fn show_gf_low(field: &mut ValueField, state: &State) {
    field.set_text_fmt(format_args!("{}%", state.settings.gf_low));
}

fn show_sac(field: &mut ValueField, state: &State) {
    field.set_text_fmt(format_args!("{} L/min", state.settings.sac_l_min));
}

fn show_backlight(field: &mut ValueField, state: &State) {
    field.set_text_fmt(format_args!("{}%", state.settings.backlight_percent));
}

fn show_screen_off(field: &mut ValueField, state: &State) {
    match state.settings.screen_off_s {
        0 => field.set_text_fmt(format_args!("Never")),
        seconds if seconds < 60 => field.set_text_fmt(format_args!("{} s", seconds)),
        seconds => field.set_text_fmt(format_args!("{} min", seconds / 60)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::mock_display::MockDisplay;

    #[test]
    fn test_settings_scene_draws() {
        let mut scene = build_settings();
        scene.update(&State::new());
        let mut display: MockDisplay<Rgb565> = MockDisplay::new();
        display.set_allow_out_of_bounds_drawing(true);
        display.set_allow_overdraw(true);
        scene.draw(&mut display).unwrap();
        assert_ne!(display, MockDisplay::new());
    }

    #[test]
    fn test_editing_day_through_scene() {
        let mut scene = build_settings();
        let mut state = State::new();
        state.datetime.date.day = 16;
        scene.update(&state);

        // The day field is selected first
        scene.click_currently_toggled();
        assert_eq!(scene.select_next_toggleable(), Action::AdjustSetting(Setting::Day, 1));
    }
}
