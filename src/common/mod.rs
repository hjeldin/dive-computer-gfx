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

    /// Returns true if the component changed since it was last drawn.
    fn needs_redraw(&self) -> bool;

    /// Cleans up any resources used by the component.
    /// This is called when the component is being removed or when its scene becomes inactive.
    fn cleanup(&mut self) {}
}

/// An enum that represents all drawable component types.
///
/// An enum is used instead of trait objects: `DrawableComponent::draw` is generic over
/// the draw target, so the trait is not object-safe, and `no_std` without an allocator
/// has no `Box<dyn _>` to store heterogeneous components in.
pub enum Drawable {
    Background(crate::utils::background::Background),
    Button(Button),
    Compass(crate::components::compass::Compass),
    Label(crate::components::label::Label),
    DepthProfile(crate::components::depth_profile::DepthProfile),
}

impl Drawable {
    /// Returns true if the component can receive the selection highlight.
    pub fn is_selectable(&self) -> bool {
        matches!(self, Drawable::Button(_) | Drawable::Compass(_))
    }

    /// Clicks the component, returning the action it requests.
    pub fn click(&mut self) -> Action {
        match self {
            Drawable::Button(button) => button.click(),
            _ => Action::None,
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
            Drawable::DepthProfile(_) => {}
        }
    }

    fn is_toggled(&self) -> bool {
        match self {
            Drawable::Button(button) => button.is_toggled(),
            Drawable::Compass(compass) => compass.is_toggled(),
            Drawable::Background(_) => false,
            Drawable::Label(_) => false,
            Drawable::DepthProfile(_) => false,
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
            Drawable::DepthProfile(profile) => profile.initialize(),
        }
    }

    fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        match self {
            Drawable::Button(button) => button.draw(target),
            Drawable::Background(background) => background.draw(target),
            Drawable::Compass(compass) => compass.draw(target),
            Drawable::Label(label) => label.draw(target),
            Drawable::DepthProfile(profile) => profile.draw(target),
        }
    }

    fn update(&mut self, state: &State) {
        match self {
            Drawable::Button(button) => button.update(state),
            Drawable::Background(background) => background.update(state),
            Drawable::Compass(compass) => compass.update(state),
            Drawable::Label(label) => label.update(state),
            Drawable::DepthProfile(profile) => profile.update(state),
        }
    }

    fn needs_redraw(&self) -> bool {
        match self {
            Drawable::Button(button) => button.needs_redraw(),
            Drawable::Background(background) => background.needs_redraw(),
            Drawable::Compass(compass) => compass.needs_redraw(),
            Drawable::Label(label) => label.needs_redraw(),
            Drawable::DepthProfile(profile) => profile.needs_redraw(),
        }
    }

    fn set_needs_redraw(&mut self, needs_redraw: bool) {
        match self {
            Drawable::Button(button) => button.set_needs_redraw(needs_redraw),
            Drawable::Background(background) => background.set_needs_redraw(needs_redraw),
            Drawable::Compass(compass) => compass.set_needs_redraw(needs_redraw),
            Drawable::Label(label) => label.set_needs_redraw(needs_redraw),
            Drawable::DepthProfile(profile) => profile.set_needs_redraw(needs_redraw),
        }
    }

    fn cleanup(&mut self) {
        match self {
            Drawable::Button(button) => button.cleanup(),
            Drawable::Background(background) => background.cleanup(),
            Drawable::Compass(compass) => compass.cleanup(),
            Drawable::Label(label) => label.cleanup(),
            Drawable::DepthProfile(profile) => profile.cleanup(),
        }
    }
}

/// A request a component makes to the code that owns the scenes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Nothing to do.
    None,
    /// Switch to the scene at the given index.
    SwitchScene(usize),
}

pub trait Clickable {
    fn click(&mut self) -> Action;
}

/// Copies `src` into `dst`, dropping any characters that do not fit.
pub(crate) fn copy_truncated<const N: usize>(dst: &mut heapless::String<N>, src: &str) {
    dst.clear();
    for c in src.chars() {
        if dst.push(c).is_err() {
            break;
        }
    }
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

/// What the NDL cell of the dive table shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecoDisplay {
    Unknown,
    Ndl(u8),
    NdlOver99,
    Stop { depth_m: u8, minutes: u16 },
}

pub struct State {
    pub pressure: f32,
    pub temperature: f32,
    pub datetime: DateTime,
    pub battery: u8,
    /// Milliseconds since boot, set by the platform every tick.
    pub uptime_ms: u32,
    /// Milliseconds since the current dive started, set by the dive state.
    pub dive_time_ms: u32,
    pub heading: f32,      // Current heading in degrees (0-359)
    pub set_course: Option<f32>,   // Set course in degrees (0-359), None when unset
    /// Depth below the surface in metres.
    pub depth_m: f32,
    pub max_depth_m: f32,
    /// Positive when ascending.
    pub ascent_rate_m_min: f32,
    pub deco: DecoDisplay,
    /// Time to surface in minutes, `None` when unknown.
    pub tts_min: Option<u16>,
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
            uptime_ms: 0,
            dive_time_ms: 0,
            heading: 0.0,
            set_course: None,
            depth_m: 0.0,
            max_depth_m: 0.0,
            ascent_rate_m_min: 0.0,
            deco: DecoDisplay::Unknown,
            tts_min: None,
        }
    }
}

#[cfg(test)]
mod state_tests {
    use super::*;

    #[test]
    fn new_state_has_no_dive_data() {
        let state = State::new();
        assert_eq!(state.depth_m, 0.0);
        assert_eq!(state.max_depth_m, 0.0);
        assert_eq!(state.ascent_rate_m_min, 0.0);
        assert_eq!(state.deco, DecoDisplay::Unknown);
        assert_eq!(state.tts_min, None);
    }
}
