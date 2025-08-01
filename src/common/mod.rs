use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
};

use crate::components::button::Button;
use core::result::Result;
use core::marker::Sized;

/// A trait for components that can be toggled.
pub trait Toggleable {
    /// Toggles the current state of the component.
    fn toggle(&mut self, state: bool);

    /// Returns the current toggle state.
    fn is_toggled(&self) -> bool;
}

/// A trait for components that can be drawn.
pub trait DrawableComponent {
    /// Initialize the component with any required resources.
    /// This is called when the component is first created or when it becomes active.
    fn initialize(&mut self) {}

    /// Draws the component on the given draw target.
    fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error>;

    /// Updates the component based on the current state.
    /// This is called regularly to keep the component up to date.
    fn update(&mut self, state: &State);

    /// Marks whether the component needs to be redrawn.
    fn set_needs_redraw(&mut self, needs_redraw: bool);

    /// Alias for set_needs_redraw to maintain backward compatibility.
    /// This will be deprecated in future versions.
    fn setNeedsRedraw(&mut self, needs_redraw: bool) {
        self.set_needs_redraw(needs_redraw)
    }

    /// Cleans up any resources used by the component.
    /// This is called when the component is being removed or when its scene becomes inactive.
    fn cleanup(&mut self) {}
}

/// An enum that represents all drawable component types.
/// This is being replaced with a trait-based approach for better extensibility.
pub enum Drawable {
    Background(crate::utils::background::Background),
    Button(Button),
    Compass(crate::components::compass::Compass),
    Label(crate::components::label::Label),
    Plot(crate::components::plot::Plot),
}

impl Drawable {
    /// Initialize the component with any required resources.
    pub fn initialize(&mut self) {
        match self {
            Drawable::Button(button) => button.initialize(),
            Drawable::Background(background) => background.initialize(),
            Drawable::Compass(compass) => compass.initialize(),
            Drawable::Label(label) => label.initialize(),
            Drawable::Plot(plot) => plot.initialize(),
        }
    }

    /// Draw the component.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        match self {
            Drawable::Button(button) => button.draw(target),
            Drawable::Background(background) => background.draw(target),
            Drawable::Compass(compass) => compass.draw(target),
            Drawable::Label(label) => label.draw(target),
            Drawable::Plot(plot) => plot.draw(target),
        }
    }

    /// Update the component based on the current state.
    pub fn update(&mut self, state: &State) {
        match self {
            Drawable::Button(button) => button.update(state),
            Drawable::Background(background) => background.update(state),
            Drawable::Compass(compass) => compass.update(state),
            Drawable::Label(label) => label.update(state),
            Drawable::Plot(plot) => plot.update(state),
        }
    }

    /// Set whether the component needs to be redrawn.
    pub fn set_needs_redraw(&mut self, needs_redraw: bool) {
        match self {
            Drawable::Button(button) => button.set_needs_redraw(needs_redraw),
            Drawable::Background(background) => background.set_needs_redraw(needs_redraw),
            Drawable::Compass(compass) => compass.set_needs_redraw(needs_redraw),
            Drawable::Label(label) => label.set_needs_redraw(needs_redraw),
            Drawable::Plot(plot) => plot.set_needs_redraw(needs_redraw),
        }
    }

    /// Clean up any resources used by the component.
    pub fn cleanup(&mut self) {
        match self {
            Drawable::Button(button) => button.cleanup(),
            Drawable::Background(background) => background.cleanup(),
            Drawable::Compass(compass) => compass.cleanup(),
            Drawable::Label(label) => label.cleanup(),
            Drawable::Plot(plot) => plot.cleanup(),
        }
    }
}

impl Toggleable for Drawable {
    fn toggle(&mut self, state: bool) {
        match self {
            Drawable::Button(button) => button.toggle(state),
            Drawable::Background(_) => {}
            Drawable::Compass(compass) => compass.toggle(state),
            Drawable::Label(_) => {}
            Drawable::Plot(_) => {}
        }
    }

    fn is_toggled(&self) -> bool {
        match self {
            Drawable::Button(button) => button.is_toggled(),
            Drawable::Compass(compass) => compass.is_toggled(),
            Drawable::Background(_) => false,
            Drawable::Label(_) => false,
            Drawable::Plot(_) => false,
        }
    }
}

// Implementation of DrawableComponent for Drawable
// This delegates to the appropriate component based on the enum variant
impl DrawableComponent for Drawable {
    fn initialize(&mut self) {
        match self {
            Drawable::Button(button) => button.initialize(),
            Drawable::Background(background) => background.initialize(),
            Drawable::Compass(compass) => compass.initialize(),
            Drawable::Label(label) => label.initialize(),
            Drawable::Plot(plot) => plot.initialize(),
        }
    }

    fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        match self {
            Drawable::Button(button) => button.draw(target),
            Drawable::Background(background) => background.draw(target),
            Drawable::Compass(compass) => compass.draw(target),
            Drawable::Label(label) => label.draw(target),
            Drawable::Plot(plot) => plot.draw(target),
        }
    }

    fn update(&mut self, state: &State) {
        match self {
            Drawable::Button(button) => button.update(state),
            Drawable::Background(background) => background.update(state),
            Drawable::Compass(compass) => compass.update(state),
            Drawable::Label(label) => label.update(state),
            Drawable::Plot(plot) => plot.update(state),
        }
    }

    fn set_needs_redraw(&mut self, needs_redraw: bool) {
        match self {
            Drawable::Button(button) => button.set_needs_redraw(needs_redraw),
            Drawable::Background(background) => background.set_needs_redraw(needs_redraw),
            Drawable::Compass(compass) => compass.set_needs_redraw(needs_redraw),
            Drawable::Label(label) => label.set_needs_redraw(needs_redraw),
            Drawable::Plot(plot) => plot.set_needs_redraw(needs_redraw),
        }
    }

    fn cleanup(&mut self) {
        match self {
            Drawable::Button(button) => button.cleanup(),
            Drawable::Background(background) => background.cleanup(),
            Drawable::Compass(compass) => compass.cleanup(),
            Drawable::Label(label) => label.cleanup(),
            Drawable::Plot(plot) => plot.cleanup(),
        }
    }
}

/// A trait that combines DrawableComponent and Toggleable.
/// This is the trait-based replacement for the Drawable enum.
pub trait AnyDrawable: DrawableComponent + Toggleable {}

// Implement AnyDrawable for any type that implements both DrawableComponent and Toggleable
impl<T: DrawableComponent + Toggleable> AnyDrawable for T {}

/// A type-erased wrapper for AnyDrawable.
/// This allows us to store any component type in a collection without dynamic allocation.
pub struct DrawableObject {
    // The actual component, stored as a Drawable enum
    component: Drawable,
}

impl DrawableObject {
    /// Create a new DrawableObject from a Background component.
    pub fn from_background(background: crate::utils::background::Background) -> Self {
        Self { component: Drawable::Background(background) }
    }

    /// Create a new DrawableObject from a Button component.
    pub fn from_button(button: Button) -> Self {
        Self { component: Drawable::Button(button) }
    }

    /// Create a new DrawableObject from a Compass component.
    pub fn from_compass(compass: crate::components::compass::Compass) -> Self {
        Self { component: Drawable::Compass(compass) }
    }

    /// Create a new DrawableObject from a Label component.
    pub fn from_label(label: crate::components::label::Label) -> Self {
        Self { component: Drawable::Label(label) }
    }

    /// Create a new DrawableObject from a Plot component.
    pub fn from_plot(plot: crate::components::plot::Plot) -> Self {
        Self { component: Drawable::Plot(plot) }
    }

    /// Check if this object is a Button.
    pub fn is_button(&self) -> bool {
        match self.component {
            Drawable::Button(_) => true,
            _ => false,
        }
    }

    /// Get a mutable reference to the Button component, if this is a Button.
    pub fn as_button_mut(&mut self) -> Option<&mut Button> {
        match &mut self.component {
            Drawable::Button(button) => Some(button),
            _ => None,
        }
    }
}

// Implement DrawableComponent for DrawableObject by delegating to the wrapped component
impl DrawableComponent for DrawableObject {
    fn initialize(&mut self) {
        self.component.initialize();
    }

    fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        self.component.draw(target)
    }

    fn update(&mut self, state: &State) {
        self.component.update(state);
    }

    fn set_needs_redraw(&mut self, needs_redraw: bool) {
        self.component.set_needs_redraw(needs_redraw);
    }

    fn cleanup(&mut self) {
        self.component.cleanup();
    }
}

// Implement Toggleable for DrawableObject by delegating to the wrapped component
impl Toggleable for DrawableObject {
    fn toggle(&mut self, state: bool) {
        self.component.toggle(state);
    }

    fn is_toggled(&self) -> bool {
        self.component.is_toggled()
    }
}

// Note: DrawableObject automatically implements AnyDrawable through the blanket implementation
// because it implements both DrawableComponent and Toggleable


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
    pub heading: f32,      // Current heading in degrees (0-359)
    pub set_course: f32,   // Set course in degrees (0-359)
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
            heading: 0.0,
            set_course: 0.0,
        }
    }
}
