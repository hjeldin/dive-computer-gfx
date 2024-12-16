use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyleBuilder},
};
use embedded_graphics::geometry::Size;

use crate::common::DrawableComponent;


pub struct Background {
    position: Point,
    size: Size
}

impl Background {
    pub fn new(position: Point, size: Size) -> Self {
        Self {
            position,
            size
        }
    }

    pub fn draw<D: DrawTarget<Color = Rgb565>>(
        &self,
        target: &mut D,
    ) -> Result<(), D::Error> {
        // Define styles
        let border_style = PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::BLACK)
            .build();

        // Draw the button's background (rectangle)
        let rect = Rectangle::new(self.position, self.size).into_styled(border_style);
        rect.draw(target)?;

        Ok(())
    }
}

impl DrawableComponent for Background {
    fn draw<D: DrawTarget<Color = Rgb565>>(&self, target: &mut D) -> Result<(), D::Error> {
        self.draw(target)
    }
}