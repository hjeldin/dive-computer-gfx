use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::MonoTextStyle,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, Triangle},
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};
use embedded_graphics::geometry::Size;
use profont::PROFONT_9_POINT;

use crate::common::{Action, Clickable, DrawableComponent, Setting, State, Toggleable};
use core::option::Option::Some;
use core::option::Option::None;
use core::result::Result;
use core::result::Result::Ok;
use core::option::Option;

/// Width of the space on the right of the box that holds the arrow.
const ARROW_AREA: u32 = 14;

/// A box that shows the chosen option of a setting, and a list to choose another.
///
/// Clicking the selected dropdown opens the list below the box, over whatever is
/// there. Next then moves the highlight through the options, and a
/// second click closes the list and returns `Action::SelectSetting` with the
/// highlighted option. The scene redraws everything when the list closes.
pub struct Dropdown {
    pub position: Point,
    pub size: Size,
    setting: Setting,
    options: &'static [&'static str],
    /// Index of the option the state has, read by `current`.
    chosen: usize,
    current: fn(&State) -> usize,
    /// Index of the highlighted option while the list is open.
    highlighted: Option<usize>,
    is_selected: bool,
    needs_redraw: bool,
}

impl Dropdown {
    /// Creates a dropdown for `setting` with `options`.
    /// `current` reads the index of the chosen option from the state.
    pub fn new(
        position: Point,
        size: Size,
        setting: Setting,
        options: &'static [&'static str],
        current: fn(&State) -> usize,
    ) -> Self {
        Self {
            position,
            size,
            setting,
            options,
            chosen: 0,
            current,
            highlighted: None,
            is_selected: false,
            needs_redraw: false,
        }
    }

    /// Returns true while the list of options is shown.
    pub fn is_open(&self) -> bool {
        self.highlighted.is_some()
    }

    /// The name of the chosen option.
    pub fn chosen_label(&self) -> &'static str {
        self.options.get(self.chosen).copied().unwrap_or("")
    }

    /// Moves the highlight to the next option, wrapping around.
    /// Does nothing while the list is closed.
    pub fn step(&mut self) -> Action {
        if let Some(index) = self.highlighted {
            let next = (index + 1) % self.options.len();
            if next != index {
                self.highlighted = Some(next);
                self.needs_redraw = true;
            }
        }
        Action::None
    }

    /// Draws the box and, while open, the list of options below it.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        let (border, fill) = match (self.is_open(), self.is_selected) {
            (true, _) => (Rgb565::RED, Rgb565::GREEN),
            (false, true) => (Rgb565::RED, Rgb565::YELLOW),
            (false, false) => (Rgb565::BLUE, Rgb565::WHITE),
        };
        let box_area = Rectangle::new(self.position, self.size);
        draw_box(target, box_area, border, fill, self.chosen_label(), ARROW_AREA)?;

        // Arrow pointing down, or up while open
        let right = self.position.x + self.size.width as i32 - 4;
        let mid = self.position.y + self.size.height as i32 / 2;
        let (tip, base) = if self.is_open() { (mid - 3, mid + 3) } else { (mid + 3, mid - 3) };
        Triangle::new(Point::new(right - 8, base), Point::new(right, base), Point::new(right - 4, tip))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(target)?;

        if let Some(highlighted) = self.highlighted {
            for (index, label) in self.options.iter().enumerate() {
                let top = self.position.y + (index as i32 + 1) * self.size.height as i32;
                let fill = if index == highlighted { Rgb565::YELLOW } else { Rgb565::WHITE };
                let area = Rectangle::new(Point::new(self.position.x, top), self.size);
                draw_box(target, area, Rgb565::BLUE, fill, label, 0)?;
            }
        }
        Ok(())
    }
}

/// Draws a bordered box with `text` centred in it, leaving `right_margin`
/// pixels free on the right.
fn draw_box<D: DrawTarget<Color = Rgb565>>(
    target: &mut D,
    area: Rectangle,
    border: Rgb565,
    fill: Rgb565,
    text: &str,
    right_margin: u32,
) -> Result<(), D::Error> {
    let style = PrimitiveStyleBuilder::new()
        .stroke_color(border)
        .stroke_width(2)
        .fill_color(fill)
        .build();
    area.into_styled(style).draw(target)?;
    let text_width = area.size.width.saturating_sub(right_margin);
    let center = area.top_left + Point::new(text_width as i32 / 2, area.size.height as i32 / 2);
    Text::with_text_style(
        text,
        center,
        MonoTextStyle::new(&PROFONT_9_POINT, Rgb565::BLACK),
        TextStyleBuilder::new().alignment(Alignment::Center).baseline(Baseline::Middle).build(),
    )
    .draw(target)?;
    Ok(())
}

impl Toggleable for Dropdown {
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

impl Clickable for Dropdown {
    fn click(&mut self) -> Action {
        self.needs_redraw = true;
        match self.highlighted.take() {
            Some(index) => Action::SelectSetting(self.setting, index),
            None => {
                if !self.options.is_empty() {
                    self.highlighted = Some(self.chosen);
                }
                Action::None
            }
        }
    }
}

impl DrawableComponent for Dropdown {
    fn initialize(&mut self) {
        self.needs_redraw = true;
    }

    fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        self.draw(target)
    }

    fn update(&mut self, state: &State) {
        let chosen = (self.current)(state);
        if chosen != self.chosen {
            self.chosen = chosen;
            self.needs_redraw = true;
        }
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_needs_redraw(&mut self, needs_redraw: bool) {
        self.needs_redraw = needs_redraw;
    }

    fn cleanup(&mut self) {
        self.highlighted = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPTIONS: [&str; 3] = ["A", "B", "C"];

    fn dropdown() -> Dropdown {
        Dropdown::new(Point::zero(), Size::new(60, 20), Setting::UsbMode, &OPTIONS, |_| 1)
    }

    #[test]
    fn test_open_step_and_choose() {
        let mut dropdown = dropdown();
        DrawableComponent::update(&mut dropdown, &State::new());
        assert_eq!(dropdown.chosen_label(), "B");

        assert_eq!(dropdown.click(), Action::None);
        assert!(dropdown.is_open());
        dropdown.step();
        dropdown.step();
        assert_eq!(dropdown.click(), Action::SelectSetting(Setting::UsbMode, 0), "wraps past the last option");
        assert!(!dropdown.is_open());
    }

    #[test]
    fn test_step_while_closed_does_nothing() {
        let mut dropdown = dropdown();
        dropdown.step();
        assert!(!dropdown.is_open());
    }
}
