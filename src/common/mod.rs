use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
};

use crate::components::button::Button;
use core::result::Result;

/// A trait for components that can be toggled.
pub trait Toggleable {
    /// Toggles the current state of the component.
    fn toggle(&mut self, state: bool);

    /// Returns the current toggle state.
    fn is_toggled(&self) -> bool;
}

pub enum Drawable {
    Background(crate::utils::background::Background),
    Button(Button),
    Label(crate::components::label::Label),
    Plot(crate::components::plot::Plot),
}

impl Drawable {
    /// Draw the component.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        match self {
            Drawable::Button(button) => button.draw(target),
            Drawable::Background(background) => background.draw(target),
            Drawable::Label(label) => label.draw(target),
            Drawable::Plot(plot) => plot.draw(target),
        }
    }
    
    pub fn update(&mut self, state: &State) {
        match self {
            Drawable::Button(button) => button.update(state),
            Drawable::Background(background) => background.update(state),
            Drawable::Label(label) => label.update(state),
            Drawable::Plot(plot) => plot.update(state),
        }
    }
}

impl Toggleable for Drawable {
    fn toggle(&mut self, state: bool) {
        match self {
            Drawable::Button(button) => button.toggle(state),
            Drawable::Background(_) => {}
            Drawable::Label(_) => {}
            Drawable::Plot(_) => {}
        }
    }

    fn is_toggled(&self) -> bool {
        // todo!()
        true
    }
}

/// A trait for components that can be drawn.
pub trait DrawableComponent {
    // Draws the component on the given draw target.
    fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error>;
    fn update(&mut self, state: &State);
    fn setNeedsRedraw(&mut self, needs_redraw: bool);
}


pub trait Clickable {
    fn click(&mut self);
}

pub struct Time {
    pub hours: u8,
    pub minutes: u8,
}

impl Time {
    pub fn new(hours: u8, minutes: u8) -> Self {
        Self { hours, minutes }
    }
}


pub struct Date {
    pub day: u8,
    pub month: u8,
    pub year: u16,
}

impl Date {
    pub fn new(day: u8, month: u8, year: u16) -> Self {
        Self { day, month, year }
    }
}

pub struct DateTime {
    pub date: Date,
    pub time: Time,
}

pub struct State {
    pub pressure: f32,
    pub temperature: f32,
    pub datetime: DateTime,
    pub battery: u8,
}

impl State {
    pub fn new() -> Self {
        Self {
            pressure: 20.0,
            temperature: 20.0,
            datetime: DateTime {
                date: Date::new(1, 1, 2025),
                time: Time::new(22, 35),
            },
            battery: 0xff,
        }
    }
}