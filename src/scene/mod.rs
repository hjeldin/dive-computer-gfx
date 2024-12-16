use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
};
use crate::common::{Clickable, Drawable, Toggleable};
use heapless::Vec;

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
    pub fn add_component(&mut self, component: Drawable) {
        let _ = self.components.push(component);
    }

    /// Draw all components in the scene.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(&self, target: &mut D) -> Result<(), D::Error> {
        for component in &self.components {
            component.draw(target)?;
        }
        Ok(())
    }

    pub fn cleanup(&mut self) {
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

        let start_index = self.current_toggle_index.unwrap_or(usize::MAX) + 1; // Start after the current toggleable
        let len = self.components.len();

        self.components[self.current_toggle_index.unwrap()].toggle(false);

        for i in 0..len {
            let index = (start_index + i) % len; // Wrap around
            if let Drawable::Button(_) = &self.components[index] {
                // Toggle the next toggleable item
                self.components[index].toggle(true);
                self.current_toggle_index = Some(index);
                return;
            }
        }
    }

    /// Toggle the previous toggleable component in the vector.
    pub fn select_prev_toggleable(&mut self) {
        if self.components.is_empty() {
            return;
        }

        let len = self.components.len();
        let start_index = self.current_toggle_index.unwrap_or(0) as usize;

        self.components[self.current_toggle_index.unwrap()].toggle(false);

        for i in 0..len {
            let index = (start_index - i) % len; // Wrap around
            if let Drawable::Button(_) = &self.components[index] {
                // Toggle the next toggleable item
                self.components[index].toggle(true);
                self.current_toggle_index = Some(index);
                return;
            }
        }
    }

    /// Click the currently toggled element in the components vector.
    pub fn click_currently_toggled(&mut self) {
        if let Some(index) = self.current_toggle_index {
            if let Drawable::Button(button) = &mut self.components[index] {
                button.click(); // Click the currently toggled button
            }
        }
    }
   
}
