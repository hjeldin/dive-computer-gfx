use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::MonoTextStyle,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};
use embedded_graphics::geometry::Size;
use profont::PROFONT_9_POINT;

use crate::common::{Action, Clickable, DrawableComponent, Setting, State, Toggleable};
use heapless::String;
use core::option::Option::Some;
use core::option::Option::None;
use core::result::Result;
use core::result::Result::Ok;
use core::option::Option;

const MAX_TEXT_LENGTH: usize = 16;

/// A box that shows one setting and changes it.
///
/// Clicking the selected field starts editing: next then steps the setting up,
/// wrapping at its highest value, through `Action::AdjustSetting`. Clicking again
/// ends editing.
/// The field does not keep the value itself: its `on_update` callback shows the
/// value from the state, so the state stays the only copy.
pub struct ValueField {
    pub position: Point,
    pub size: Size,
    setting: Setting,
    text: String<MAX_TEXT_LENGTH>,
    on_update: Option<fn(&mut Self, &State)>,
    is_selected: bool,
    is_editing: bool,
    needs_redraw: bool,
}

impl ValueField {
    /// Creates a field that changes `setting`. Set `on_update` to show its value.
    pub fn new(position: Point, size: Size, setting: Setting) -> Self {
        Self {
            position,
            size,
            setting,
            text: String::new(),
            on_update: None,
            is_selected: false,
            is_editing: false,
            needs_redraw: false,
        }
    }

    /// Sets the callback that shows the setting's value, called on every update.
    pub fn set_on_update(&mut self, callback: fn(&mut Self, &State)) {
        self.on_update = Some(callback);
    }

    /// Formats `args` into the shown text, truncating it to fit.
    /// Marks the field for redraw only if the text changed.
    pub fn set_text_fmt(&mut self, args: core::fmt::Arguments) {
        let mut buf = [0u8; MAX_TEXT_LENGTH];
        if let Ok(text) = format_no_std::show(&mut buf, args) {
            if text != self.text.as_str() {
                crate::common::copy_truncated(&mut self.text, text);
                self.needs_redraw = true;
            }
        }
    }

    /// The text currently shown.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns true while next changes the value.
    pub fn is_editing(&self) -> bool {
        self.is_editing
    }

    /// Steps the setting up while editing.
    pub fn step(&mut self) -> Action {
        if !self.is_editing {
            return Action::None;
        }
        Action::AdjustSetting(self.setting, 1)
    }

    /// Draws the field on the given draw target.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        let (border, fill) = match (self.is_editing, self.is_selected) {
            (true, _) => (Rgb565::RED, Rgb565::GREEN),
            (false, true) => (Rgb565::RED, Rgb565::YELLOW),
            (false, false) => (Rgb565::BLUE, Rgb565::WHITE),
        };
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(border)
            .stroke_width(2)
            .fill_color(fill)
            .build();
        Rectangle::new(self.position, self.size).into_styled(style).draw(target)?;

        let center = self.position + Point::new(self.size.width as i32 / 2, self.size.height as i32 / 2);
        Text::with_text_style(
            &self.text,
            center,
            MonoTextStyle::new(&PROFONT_9_POINT, Rgb565::BLACK),
            TextStyleBuilder::new().alignment(Alignment::Center).baseline(Baseline::Middle).build(),
        )
        .draw(target)?;
        Ok(())
    }
}

impl Toggleable for ValueField {
    fn toggle(&mut self, state: bool) {
        if self.is_selected != state {
            self.is_selected = state;
            self.needs_redraw = true;
        }
    }

    fn is_toggled(&self) -> bool {
        self.is_selected
    }
}

impl Clickable for ValueField {
    fn click(&mut self) -> Action {
        self.is_editing = !self.is_editing;
        self.needs_redraw = true;
        Action::None
    }
}

impl DrawableComponent for ValueField {
    fn initialize(&mut self) {
        self.needs_redraw = true;
    }

    fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        self.draw(target)
    }

    fn update(&mut self, state: &State) {
        if let Some(callback) = self.on_update {
            callback(self, state);
        }
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_needs_redraw(&mut self, needs_redraw: bool) {
        self.needs_redraw = needs_redraw;
    }

    fn cleanup(&mut self) {
        self.on_update = None;
        self.is_editing = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field() -> ValueField {
        let mut field = ValueField::new(Point::zero(), Size::new(40, 20), Setting::GfHigh);
        field.set_on_update(|field, state| field.set_text_fmt(format_args!("{}%", state.settings.gf_high)));
        field
    }

    #[test]
    fn test_steps_only_while_editing() {
        let mut field = field();
        assert_eq!(field.step(), Action::None);
        field.click();
        assert!(field.is_editing());
        assert_eq!(field.step(), Action::AdjustSetting(Setting::GfHigh, 1));
        field.click();
        assert!(!field.is_editing());
    }

    #[test]
    fn test_update_shows_state() {
        let mut field = field();
        let mut state = State::new();
        state.settings.gf_high = 85;
        DrawableComponent::update(&mut field, &state);
        assert_eq!(field.text(), "85%");
        field.set_needs_redraw(false);
        DrawableComponent::update(&mut field, &state);
        assert!(!field.needs_redraw(), "same text, no redraw");
    }
}
