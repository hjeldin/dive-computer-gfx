use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::Rectangle,
    text::{Text, TextStyleBuilder},
    mono_font::{ascii::FONT_6X9, MonoTextStyle},
};
use embedded_graphics::geometry::Size;

use crate::common::DrawableComponent;
use heapless::String;

const MAX_LABEL_LENGTH: usize = 32;

/// Represents a button component that can be toggled and drawn.
pub struct Label {
    pub position: Point,
    pub size: Size,
    pub label: String<MAX_LABEL_LENGTH>,
}

impl Label {
    /// Creates a new `Button` instance.
    pub fn new(position: Point, size: Size, label: &str) -> Self {
        let mut label_str: String<MAX_LABEL_LENGTH> = String::new();
        label_str.push_str(label).unwrap();
        Self {
            position,
            size,
            label: label_str,
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
        let text_style = MonoTextStyle::new(&FONT_6X9, Rgb565::BLACK);
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

impl DrawableComponent for Label {
    fn draw<D: DrawTarget<Color = Rgb565>>(&self, target: &mut D) -> Result<(), D::Error> {
        self.draw(target)
    }
}