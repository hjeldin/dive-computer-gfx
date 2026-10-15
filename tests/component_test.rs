use stm_graphics::components::button::Button;
use stm_graphics::components::label::Label;
use stm_graphics::{Action, Clickable, DrawableComponent, State, Toggleable};
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mock_display::MockDisplay;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use profont::PROFONT_9_POINT;

fn test_button() -> Button {
    Button::new(Point::new(0, 0), Size::new(10, 10), "Test Button")
}

/// A mock display that accepts the button label spilling past its 10x10 bounds.
fn permissive_display() -> MockDisplay<Rgb565> {
    let mut display = MockDisplay::new();
    display.set_allow_out_of_bounds_drawing(true);
    display.set_allow_overdraw(true);
    display
}

#[test]
fn test_button_creation() {
    let button = test_button();

    assert_eq!(button.position, Point::new(0, 0));
    assert_eq!(button.size, Size::new(10, 10));
    assert_eq!(button.label.as_str(), "Test Button");
    assert!(!button.is_selected);
    assert!(button.on_click.is_none());
}

#[test]
fn test_button_label_is_truncated() {
    let long = "A label that is much longer than thirty-two bytes";
    let button = Button::new(Point::new(0, 0), Size::new(10, 10), long);
    assert_eq!(button.label.as_str(), &long[..32]);
}

#[test]
fn test_label_text_is_truncated() {
    let style = MonoTextStyle::new(&PROFONT_9_POINT, Rgb565::WHITE);
    let mut label = Label::new(Point::new(0, 0), Size::new(10, 10), "", style);
    label.set_label_text("0123456789012345678901234567890123456789");
    assert_eq!(label.label.len(), 32);
}

#[test]
fn test_button_toggle() {
    let mut button = test_button();
    assert!(!button.is_toggled());

    button.toggle(true);
    assert!(button.is_toggled());

    button.toggle(false);
    assert!(!button.is_toggled());
}

#[test]
fn test_button_click() {
    let mut button = test_button();
    assert_eq!(button.click(), Action::None);

    button.set_on_click(|button| {
        button.toggle(true);
        Action::SwitchScene(1)
    });

    assert_eq!(button.click(), Action::SwitchScene(1));
    assert!(button.is_toggled());
}

#[test]
fn test_button_draw() {
    let mut button = test_button();
    button.initialize();

    let mut display = permissive_display();
    assert!(button.draw(&mut display).is_ok());
    assert_ne!(display, MockDisplay::new());
}

#[test]
fn test_button_update_leaves_button_unchanged() {
    let mut button = test_button();
    button.update(&State::new());
    assert_eq!(button.label.as_str(), "Test Button");
    assert!(!button.is_toggled());
}

#[test]
fn test_button_cleanup() {
    let mut button = test_button();
    button.set_on_click(|_| Action::None);
    assert!(button.on_click.is_some());

    button.cleanup();
    assert!(button.on_click.is_none());
}

#[test]
fn test_button_contains() {
    let button = test_button();

    assert!(button.contains(Point::new(0, 0)));
    assert!(button.contains(Point::new(5, 5)));
    assert!(button.contains(Point::new(9, 9)));

    assert!(!button.contains(Point::new(-1, 0)));
    assert!(!button.contains(Point::new(0, -1)));
    assert!(!button.contains(Point::new(10, 0)));
    assert!(!button.contains(Point::new(0, 10)));
}

#[test]
fn test_drawable_component_lifecycle() {
    let mut button = test_button();
    button.set_on_click(|_| Action::None);

    button.initialize();
    assert!(button.needs_redraw);

    let mut display = permissive_display();
    assert!(button.draw(&mut display).is_ok());

    button.update(&State::new());
    button.set_needs_redraw(false);
    assert!(!button.needs_redraw);

    button.cleanup();
    assert!(button.on_click.is_none());
}
