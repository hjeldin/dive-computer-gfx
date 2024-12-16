use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyleBuilder},
    text::{Text, TextStyleBuilder},
    mono_font::{ascii::FONT_6X9, MonoTextStyle},
};
use embedded_graphics::geometry::Size;

use crate::common::DrawableComponent;

/// Represents the top status bar component.
pub struct StatusBar {
    pub time_hours: u8,
    pub time_minutes: u8
}

impl StatusBar {
    pub fn new(time_hours: u8, time_minutes: u8) -> Self {
        Self {
            time_hours,
            time_minutes
        }
    }

    /// Draws the button on the given draw target.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(
        &self,
        target: &mut D,
    ) -> Result<(), D::Error> {
        // Style objects
        let text_style = MonoTextStyle::new(&FONT_6X9, Rgb565::WHITE);
        
        // Draw a filled square
        let fill = PrimitiveStyleBuilder::new().fill_color(Rgb565::CSS_SLATE_GRAY).build();
        Rectangle::new(Point::new(0, 0), Size::new(320, 16))
        .into_styled(fill)
        .draw(target)?;
        
        let mut buf = [0u8; 5];
        let text = format_no_std::show(&mut buf, format_args!("{:02}:{:02}", self.time_hours, self.time_minutes)).unwrap_or("xx:xx");
        Text::with_text_style(
            &text,
            Point::new(160, 10),
            text_style,
            TextStyleBuilder::new().alignment(embedded_graphics::text::Alignment::Center).build(),
        )
        .draw(target)?;

        Ok(())
    }
}

impl DrawableComponent for StatusBar {
    fn draw<D: DrawTarget<Color = Rgb565>>(
        &self,
        target: &mut D,
    ) -> Result<(), D::Error> {
        self.draw(target)
    }
}