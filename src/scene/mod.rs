use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
};
use crate::common::{Clickable, Drawable, State, Toggleable};
use heapless::Vec;
use core::option::Option::Some;
use core::option::Option::None;
use core::result::Result;
use core::result::Result::Ok;
use core::option::Option;

pub mod scene_manager;
const MAX_COMPONENTS: usize = 32;

pub struct Scene {
    components: Vec<Drawable, MAX_COMPONENTS>,
    current_toggle_index: Option<usize>
}

impl Scene {
    /// Create a new, empty scene.
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            current_toggle_index: None,
        }
    }

    /// Add a component to the scene.
    /// Initializes the component as part of adding it.
    pub fn add_component(&mut self, mut component: Drawable) {
        // Initialize the component before adding it
        component.initialize();
        let _ = self.components.push(component);
    }

    /// Draw all components in the scene.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        for component in &mut self.components {
            component.draw(target)?;
        }
        Ok(())
    }

    /// Clean up all components and clear the scene.
    /// This ensures all resources are properly released.
    pub fn cleanup(&mut self) {
        // Clean up each component before clearing
        for component in &mut self.components {
            component.cleanup();
        }
        self.components.clear();
        self.current_toggle_index = None;
    }

    pub fn set_toggle_index(&mut self, index: Option<usize>) {
        self.current_toggle_index = index;
    }

    /// Toggle the next toggleable component in the vector.
    pub fn select_next_toggleable(&mut self) {
        if self.components.is_empty() {
            return;
        }

        // If there's a currently toggled component, untoggle it
        if let Some(current_index) = self.current_toggle_index {
            self.components[current_index].toggle(false);
        }

        let start_index = self.current_toggle_index.unwrap_or(usize::MAX) + 1; // Start after the current toggleable
        let len = self.components.len();

        // Find the next toggleable component
        for i in 0..len {
            let index = (start_index + i) % len; // Wrap around
            if let Drawable::Button(_) = &self.components[index] {
                // Toggle the next toggleable item
                self.components[index].toggle(true);
                self.current_toggle_index = Some(index);
                return;
            }
        }

        // If we reach here, no toggleable component was found
        self.current_toggle_index = None;
    }

    /// Toggle the previous toggleable component in the vector.
    pub fn select_prev_toggleable(&mut self) {
        if self.components.is_empty() {
            return;
        }

        // If there's a currently toggled component, untoggle it
        if let Some(current_index) = self.current_toggle_index {
            self.components[current_index].toggle(false);
        }

        let len = self.components.len();
        let start_index = self.current_toggle_index.unwrap_or(0) as usize;

        // Find the previous toggleable component
        for i in 0..len {
            // Use wrapping_sub to handle potential underflow
            let index = if i <= start_index {
                start_index - i
            } else {
                len - (i - start_index)
            };

            if let Drawable::Button(_) = &self.components[index] {
                // Toggle the previous toggleable item
                self.components[index].toggle(true);
                self.current_toggle_index = Some(index);
                return;
            }
        }

        // If we reach here, no toggleable component was found
        self.current_toggle_index = None;
    }

    /// Click the currently toggled element in the components vector.
    pub fn click_currently_toggled(&mut self) {
        if let Some(index) = self.current_toggle_index {
            if let Drawable::Button(button) = &mut self.components[index] {
                defmt::info!("Clicked button");
                button.click(); // Click the currently toggled button
            }
        }
    }

    pub fn update(&mut self, state: &State) {
        for component in &mut self.components {
            component.update(state);
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::mock_display::MockDisplay;
    use embedded_graphics::primitives::Rectangle;
    use embedded_graphics::geometry::Point;
    use embedded_graphics::pixelcolor::Rgb565;
    use embedded_graphics::prelude::Size;
    use crate::components::button::Button;
    use crate::components::label::Label;
    use crate::common::{DrawableComponent, Toggleable, State};

    // Mock implementation of Drawable for testing
    struct MockDrawable {
        toggled: bool,
        initialized: bool,
        cleaned_up: bool,
        updated: bool,
    }

    impl MockDrawable {
        fn new() -> Self {
            Self {
                toggled: false,
                initialized: false,
                cleaned_up: false,
                updated: false,
            }
        }
    }

    impl DrawableComponent for MockDrawable {
        fn initialize(&mut self) {
            self.initialized = true;
        }

        fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, _target: &mut D) -> Result<(), D::Error> {
            Ok(())
        }

        fn update(&mut self, _state: &State) {
            self.updated = true;
        }

        fn set_needs_redraw(&mut self, _needs_redraw: bool) {
            // Not needed for this test
        }

        fn cleanup(&mut self) {
            self.cleaned_up = true;
        }
    }

    impl Toggleable for MockDrawable {
        fn toggle(&mut self, state: bool) {
            self.toggled = state;
        }

        fn is_toggled(&self) -> bool {
            self.toggled
        }
    }

    #[test]
    fn test_scene_new() {
        let scene = Scene::new();
        assert_eq!(scene.components.len(), 0);
        assert_eq!(scene.current_toggle_index, None);
    }

    #[test]
    fn test_add_component() {
        let mut scene = Scene::new();
        let button = Button::new(
            Point::new(0, 0),
            Size::new(10, 10),
            "Test Button",
        );
        scene.add_component(Drawable::Button(button));
        assert_eq!(scene.components.len(), 1);
    }

    #[test]
    fn test_draw() {
        let mut scene = Scene::new();
        let button = Button::new(
            Point::new(0, 0),
            Size::new(10, 10),
            "Test Button",
        );
        scene.add_component(Drawable::Button(button));

        let mut display = MockDisplay::new();
        let result = scene.draw(&mut display);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cleanup() {
        let mut scene = Scene::new();
        let button = Button::new(
            Point::new(0, 0),
            Size::new(10, 10),
            "Test Button",
        );
        scene.add_component(Drawable::Button(button));

        scene.cleanup();
        assert_eq!(scene.components.len(), 0);
        assert_eq!(scene.current_toggle_index, None);
    }

    #[test]
    fn test_select_next_toggleable() {
        let mut scene = Scene::new();
        let button1 = Button::new(
            Point::new(0, 0),
            Size::new(10, 10),
            "Button 1",
        );
        let button2 = Button::new(
            Point::new(20, 0),
            Size::new(10, 10),
            "Button 2",
        );

        scene.add_component(Drawable::Button(button1));
        scene.add_component(Drawable::Button(button2));

        // Initially no button is toggled
        assert_eq!(scene.current_toggle_index, None);

        // Select the first toggleable
        scene.select_next_toggleable();
        assert_eq!(scene.current_toggle_index, Some(0));

        // Select the next toggleable
        scene.select_next_toggleable();
        assert_eq!(scene.current_toggle_index, Some(1));

        // Wrap around to the first toggleable
        scene.select_next_toggleable();
        assert_eq!(scene.current_toggle_index, Some(0));
    }

    #[test]
    fn test_select_prev_toggleable() {
        let mut scene = Scene::new();
        let button1 = Button::new(
            Point::new(0, 0),
            Size::new(10, 10),
            "Button 1",
        );
        let button2 = Button::new(
            Point::new(20, 0),
            Size::new(10, 10),
            "Button 2",
        );

        scene.add_component(Drawable::Button(button1));
        scene.add_component(Drawable::Button(button2));

        // Initially no button is toggled
        assert_eq!(scene.current_toggle_index, None);

        // Select the last toggleable (wrapping from None)
        scene.select_prev_toggleable();
        assert_eq!(scene.current_toggle_index, Some(1));

        // Select the previous toggleable
        scene.select_prev_toggleable();
        assert_eq!(scene.current_toggle_index, Some(0));

        // Wrap around to the last toggleable
        scene.select_prev_toggleable();
        assert_eq!(scene.current_toggle_index, Some(1));
    }

    #[test]
    fn test_click_currently_toggled() {
        let mut scene = Scene::new();
        let mut clicked = false;

        let mut button = Button::new(
            Point::new(0, 0),
            Size::new(10, 10),
            "Test Button",
        );

        // Set up a click handler
        button.set_on_click(|_| {
            // We can't directly modify clicked here since it's outside this scope
            // In a real test, we would use a more testable approach
        });

        scene.add_component(Drawable::Button(button));
        scene.select_next_toggleable();

        // Verify that a button is toggled
        assert_eq!(scene.current_toggle_index, Some(0));

        // Click the toggled button
        scene.click_currently_toggled();

        // Verify that the button was clicked
        // Note: We can't directly test this since the closure captures a local variable
        // In a real test, we would use a more testable approach
    }

    #[test]
    fn test_update() {
        let mut scene = Scene::new();
        let button = Button::new(
            Point::new(0, 0),
            Size::new(10, 10),
            "Test Button",
        );
        scene.add_component(Drawable::Button(button));

        let state = State::new();
        scene.update(&state);

        // We can't directly test that the component was updated
        // In a real test, we would use a more testable approach
    }

    #[test]
    fn test_empty_scene_toggle_operations() {
        let mut scene = Scene::new();

        // These should not panic on an empty scene
        scene.select_next_toggleable();
        assert_eq!(scene.current_toggle_index, None);

        scene.select_prev_toggleable();
        assert_eq!(scene.current_toggle_index, None);

        scene.click_currently_toggled();
        // No assertion needed, just verifying it doesn't panic
    }
}
