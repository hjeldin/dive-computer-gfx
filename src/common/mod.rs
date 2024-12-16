use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
};

use crate::components::button::Button;

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
}

impl Drawable {
    /// Draw the component.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(&self, target: &mut D) -> Result<(), D::Error> {
        match self {
            Drawable::Button(button) => button.draw(target),
            Drawable::Background(background) => background.draw(target),
            Drawable::Label(label) => label.draw(target),
        }
    }
}

impl Toggleable for Drawable {
    fn toggle(&mut self, state: bool) {
        match self {
            Drawable::Button(button) => button.toggle(state),
            Drawable::Background(_) => {}
            Drawable::Label(_) => {}
        }
    }

    fn is_toggled(&self) -> bool {
        todo!()
    }
}

/// A trait for components that can be drawn.
pub trait DrawableComponent {
    // Draws the component on the given draw target.
    fn draw<D: DrawTarget<Color = Rgb565>>(&self, target: &mut D) -> Result<(), D::Error>;
}


pub trait Clickable {
    fn click(&mut self);
}