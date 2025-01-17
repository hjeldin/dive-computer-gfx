use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
};
use embedded_graphics::geometry::Size;

use crate::common::DrawableComponent;
use core::option::Option;
use core::option::Option::Some;
use core::result::*;
use core::result::Result::Ok;


pub struct Background {
    position: Point,
    size: Size,
    style: Option<PrimitiveStyle<Rgb565>>
}

impl Background {
    pub fn new(position: Point, size: Size, style: Option<PrimitiveStyle<Rgb565>>) -> Self {
        Self {
            position,
            size,
            style
        }
    }

    pub fn draw<D: DrawTarget<Color = Rgb565>>(
        &mut self,
        target: &mut D,
    ) -> Result<(), D::Error> {
        // Define style

        if let Some(style) = self.style {
            let rect = Rectangle::new(self.position, self.size).into_styled(style);
            rect.draw(target)?;
            return Ok(())
        } else {
            let border_style = PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::BLACK)
                .build();
    
            // Draw the button's background (rectangle)
            let rect = Rectangle::new(self.position, self.size).into_styled(border_style);
            rect.draw(target)?;
        }


        Ok(())
    }
}

impl DrawableComponent for Background {
    fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        defmt::info!("Bkg draw");
        self.draw(target)
    }
    
    fn update(&mut self, state: &crate::common::State) {
        // todo!()
    }

    fn setNeedsRedraw(&mut self, needs_redraw: bool) {
        // todo!()
    }
    
}