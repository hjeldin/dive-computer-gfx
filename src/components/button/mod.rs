use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyleBuilder},
    text::{Text, TextStyleBuilder},
    mono_font::{ascii::FONT_6X9, MonoTextStyle},
};
use embedded_graphics::geometry::Size;

use crate::common::{Clickable, DrawableComponent, Toggleable};
use heapless::String;

const MAX_LABEL_LENGTH: usize = 32;

/// Represents a button component that can be toggled and drawn.
pub struct Button {
    pub position: Point,
    pub size: Size,
    pub label: String<MAX_LABEL_LENGTH>,
    pub is_selected: bool,
    pub on_click: fn(),
}

impl Button {
    /// Creates a new `Button` instance.
    pub fn new(position: Point, size: Size, label: &str, on_click: fn()) -> Self {
        let mut label_str: String<MAX_LABEL_LENGTH> = String::new();
        label_str.push_str(label).unwrap();
        Self {
            position,
            size,
            label: label_str,
            is_selected: false,
            on_click: on_click,
        }
    }

    /// Checks if a given point is inside the button's bounds.
    pub fn contains(&self, point: Point) -> bool {
        let bounds = Rectangle::new(self.position, self.size);
        bounds.contains(point)
    }

    /// Draws the button on the given draw target.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(
        &self,
        target: &mut D,
    ) -> Result<(), D::Error> {
        // Define styles
        let border_style = PrimitiveStyleBuilder::new()
            .stroke_color(if self.is_selected { Rgb565::RED } else { Rgb565::BLUE })
            .stroke_width(2)
            .fill_color(if self.is_selected { Rgb565::YELLOW } else { Rgb565::WHITE })
            .build();

        let text_style = MonoTextStyle::new(&FONT_6X9, Rgb565::BLACK);

        // Draw the button's background (rectangle)
        let rect = Rectangle::new(self.position, self.size).into_styled(border_style);
        rect.draw(target)?;

        // Calculate text position (centered in the button)
        let text_position = self.position + Point::new(self.size.width as i32 / 2, self.size.height as i32 / 2);

        // Draw the label
        Text::with_text_style(
            &self.label,
            text_position,
            text_style,
            TextStyleBuilder::new().alignment(embedded_graphics::text::Alignment::Center).build(),
        )
        .draw(target)?;

        Ok(())
    }
}

/// Implement `Toggleable` for `Button`.
impl Toggleable for Button {
    fn toggle(&mut self, state: bool) {
        self.is_selected = state;
    }

    fn is_toggled(&self) -> bool {
        self.is_selected
    }
}

impl DrawableComponent for Button {
    fn draw<D: DrawTarget<Color = Rgb565>>(&self, target: &mut D) -> Result<(), D::Error> {
        self.draw(target)
    }
}

impl Clickable for Button {
    fn click(&mut self) {
        (self.on_click)();
    }
}
