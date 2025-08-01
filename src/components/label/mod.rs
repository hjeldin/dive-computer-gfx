use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::*,
    text::{Text, TextStyleBuilder},
    mono_font::MonoTextStyle,
};
use embedded_graphics::geometry::Size;

use crate::common::{DrawableComponent, State};
use heapless::String;
use core::option::Option::Some;
use core::option::Option::None;
use core::result::Result;
use core::result::Result::Ok;
use core::option::Option;

const MAX_LABEL_LENGTH: usize = 32;

/// Represents a label component that can be updated and drawn.
pub struct Label {
    pub position: Point,
    pub size: Size,
    pub label: String<MAX_LABEL_LENGTH>,
    on_update: Option<fn(&mut Self, &State)>, // Function pointer
    pub style: MonoTextStyle<'static, Rgb565>,
    pub needs_redraw: bool,
}

impl Label {
    /// Creates a new `Label` instance.
    pub fn new(position: Point, size: Size, label: &str, style: MonoTextStyle<'static, Rgb565>) -> Self {
        let mut label_str: String<MAX_LABEL_LENGTH> = String::new();
        label_str.push_str(label).unwrap();
        Self {
            position,
            size,
            label: label_str,
            on_update: None, // No callback set initially
            style,
            needs_redraw: false,
        }
    }

    /// Sets the `on_update` function pointer after creation.
    pub fn set_on_update(&mut self, callback: fn(&mut Self, &State)) {
        self.on_update = Some(callback);
    }

    /// Calls the `on_update` function if it exists.
    pub fn update(&mut self, state: &State) {
        if let Some(callback) = self.on_update {
            callback(self, state);
        }
    }

    /// Updates the label's text.
    pub fn set_label_text(&mut self, new_text: &str) {
        self.label.clear();
        self.label.push_str(new_text).unwrap();
    }

    /// Draws the label on the given draw target.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(
        &mut self,
        target: &mut D,
    ) -> Result<(), D::Error> {
        let text_position =
            self.position + Point::new(self.size.width as i32 / 2, self.size.height as i32 / 2);

        Text::with_text_style(
            &self.label,
            text_position,
            self.style,
            TextStyleBuilder::new()
                .alignment(embedded_graphics::text::Alignment::Center)
                .build(),
        )
        .draw(target)?;

        Ok(())
    }
}

impl DrawableComponent for Label {
    fn initialize(&mut self) {
        // Initialize any resources needed by the label
        self.needs_redraw = true; // Mark for initial draw
    }

    fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        defmt::info!("Label draw");
        self.draw(target)
    }

    fn update(&mut self, state: &State) {
        self.update(state);
    }

    fn set_needs_redraw(&mut self, needs_redraw: bool) {
        self.needs_redraw = needs_redraw;
    }

    fn cleanup(&mut self) {
        // Release any resources used by the label
        self.on_update = None;
    }
}
