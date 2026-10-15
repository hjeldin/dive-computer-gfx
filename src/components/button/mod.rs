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

use crate::common::{copy_truncated, Action, Clickable, DrawableComponent, State, Toggleable};
use heapless::String;
use core::result::Result;
use core::result::Result::Ok;

const MAX_LABEL_LENGTH: usize = 32;

/// Represents a button component that can be toggled and drawn.
pub struct Button {
    pub position: Point,
    pub size: Size,
    pub label: String<MAX_LABEL_LENGTH>,
    pub is_selected: bool,
    pub on_click: Option<fn(&mut Self) -> Action>,
    pub needs_redraw: bool,
}

impl Button {
    /// Creates a new `Button` instance.
    pub fn new(position: Point, size: Size, label: &str) -> Self {
        let mut label_str: String<MAX_LABEL_LENGTH> = String::new();
        copy_truncated(&mut label_str, label);
        Self {
            position,
            size,
            label: label_str,
            is_selected: false,
            on_click: None,
            needs_redraw: false,
        }
    }

    pub fn set_on_click(&mut self, callback: fn(&mut Self) -> Action) {
        self.on_click = Some(callback);
    }

    /// Checks if a given point is inside the button's bounds.
    pub fn contains(&self, point: Point) -> bool {
        let bounds = Rectangle::new(self.position, self.size);
        bounds.contains(point)
    }

    /// Draws the button on the given draw target.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(
        &mut self,
        target: &mut D,
    ) -> Result<(), D::Error> {
        defmt::info!("Button draw");
        // Define styles
        let border_style = PrimitiveStyleBuilder::new()
            .stroke_color(if self.is_selected { Rgb565::RED } else { Rgb565::BLUE })
            .stroke_width(2)
            .fill_color(if self.is_selected { Rgb565::YELLOW } else { Rgb565::WHITE })
            .build();

        let text_style = MonoTextStyle::new(&PROFONT_9_POINT, Rgb565::BLACK);

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
        if self.is_selected != state {
            self.is_selected = state;
            self.needs_redraw = true;
        }
    }

    fn is_toggled(&self) -> bool {
        self.is_selected
    }
}

impl DrawableComponent for Button {
    fn initialize(&mut self) {
        // Initialize any resources needed by the button
        self.needs_redraw = true; // Mark for initial draw
    }

    fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        self.draw(target)
    }

    fn update(&mut self, _state: &State) {
        // Update button state based on the current application state
        // Currently empty, but could be implemented for dynamic buttons
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_needs_redraw(&mut self, needs_redraw: bool) {
        self.needs_redraw = needs_redraw;
    }

    fn cleanup(&mut self) {
        // Release any resources used by the button
        // For Button, we might want to clear callbacks
        self.on_click = None;
    }
}

impl Clickable for Button {
    fn click(&mut self) -> Action {
        match self.on_click {
            Some(callback) => callback(self),
            None => Action::None,
        }
    }
}
