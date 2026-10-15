use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyleBuilder},
    text::{Text, TextStyleBuilder},
    mono_font::MonoTextStyle,
};
use embedded_graphics::geometry::Size;
use profont::PROFONT_9_POINT;
use core::format_args;
use core::result::*;
use core::result::Result::Ok;
use crate::common::DrawableComponent;

/// Represents the top status bar component.
pub struct StatusBar {
    pub time_hours: u8,
    pub time_minutes: u8,
    pub needs_redraw: bool
}

impl StatusBar {
    pub fn new(time_hours: u8, time_minutes: u8) -> Self {
        Self {
            time_hours,
            time_minutes,
            needs_redraw: true
        }
    }

    /// Draws the button on the given draw target.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(
        &mut self,
        target: &mut D,
    ) -> Result<(), D::Error> {
        // Style objects
        let text_style = MonoTextStyle::new(&PROFONT_9_POINT, Rgb565::WHITE);

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
    fn initialize(&mut self) {
        // Initialize any resources needed by the status bar
        self.needs_redraw = true; // Mark for initial draw
    }

    fn draw<D: DrawTarget<Color = Rgb565>>(
        &mut self,
        target: &mut D,
    ) -> Result<(), D::Error> {
        self.draw(target)
    }

    fn update(&mut self, state: &crate::common::State) {
        // Update time from state if needed
        let (hours, minutes) = (state.datetime.time.hours, state.datetime.time.minutes);
        if (hours, minutes) != (self.time_hours, self.time_minutes) {
            self.time_hours = hours;
            self.time_minutes = minutes;
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
        // Release any resources used by the status bar
        // Currently no resources to clean up
    }
}
