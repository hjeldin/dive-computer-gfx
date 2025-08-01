// use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*
};

use heapless::Vec;
use static_cell::StaticCell;
// use once_cell::sync::OnceCell;
use core::option::Option::Some;
use core::option::Option::None;
use core::result::Result;
use core::result::Result::Ok;
use core::option::Option;

// use spin::Mutex;

use super::Scene;

const MAX_SCENES_COUNT: usize = 5;
/// Manages multiple scenes and handles the currently active scene.
pub struct SceneManager {
    scenes: Vec<Scene, MAX_SCENES_COUNT>,
    current_scene_index: Option<usize>, // None if no scene is active
}
// static INSTANCE: StaticCell<Mutex<ThreadModeRawMutex, SceneManager>> = StaticCell::new();
pub static mut INSTANCE: SceneManager = SceneManager::new();
impl SceneManager {
    /// Create a new `SceneManager`.
    pub const fn new() -> Self {
        Self {
            scenes: Vec::new(),
            current_scene_index: None,
        }
    }

    // pub fn instance() -> &'static Mutex<ThreadModeRawMutex, SceneManager> {
    //     INSTANCE.init(Mutex::new(SceneManager::new()))
    // }

    pub fn get_active_scene(&mut self) -> Option<&mut Scene> {
        if let Some(index) = self.current_scene_index {
            self.scenes.get_mut(index)
        } else {
            None
        }
    }

    /// Add a new scene to the manager.
    pub fn add_scene(&mut self, scene: Scene) {
        let _ = self.scenes.push(scene);
        if self.current_scene_index.is_none() {
            // Automatically set the first added scene as the active one.
            self.current_scene_index = Some(0);
        }
    }

    /// Switch to a different scene by index.
    /// Properly cleans up the previous scene and initializes the new one.
    pub fn switch_scene(&mut self, index: usize) {
        if index < self.scenes.len() {
            // Clean up the current scene if there is one
            if let Some(current_index) = self.current_scene_index {
                if let Some(current_scene) = self.scenes.get_mut(current_index) {
                    current_scene.cleanup();
                }
            }

            // Set the new scene as active
            self.current_scene_index = Some(index);

            // Initialize components in the new scene if needed
            // This is already handled by Scene::add_component, but we could add additional initialization here
        } else {
            // defmt::error!("Invalid scene index: {}", index);
        }
    }

    /// Update the currently active scene.
    pub fn update_active_scene(&mut self) {
        if let Some(index) = self.current_scene_index {
            if let Some(_scene) = self.scenes.get_mut(index) {
                // Perform updates specific to the active scene (e.g., toggling or clicking items).
                // scene.toggle_next_toggleable(); // Example of scene-specific updates
            }
        }
    }

    /// Draw the currently active scene to the display.
    pub fn draw_active_scene<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if let Some(index) = self.current_scene_index {
            if let Some(scene) = self.scenes.get_mut(index) {
                scene.draw(target)?; // Delegate drawing to the active scene
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::mock_display::MockDisplay;
    use crate::common::State;
    use crate::components::button::Button;
    use embedded_graphics::geometry::{Point, Size};

    #[test]
    fn test_scene_manager_new() {
        let manager = SceneManager::new();
        assert_eq!(manager.scenes.len(), 0);
        assert_eq!(manager.current_scene_index, None);
    }

    #[test]
    fn test_add_scene() {
        let mut manager = SceneManager::new();
        let scene = Scene::new();

        manager.add_scene(scene);

        assert_eq!(manager.scenes.len(), 1);
        assert_eq!(manager.current_scene_index, Some(0)); // First scene should be active
    }

    #[test]
    fn test_add_multiple_scenes() {
        let mut manager = SceneManager::new();

        // Add first scene
        let scene1 = Scene::new();
        manager.add_scene(scene1);

        // Add second scene
        let scene2 = Scene::new();
        manager.add_scene(scene2);

        assert_eq!(manager.scenes.len(), 2);
        assert_eq!(manager.current_scene_index, Some(0)); // First scene should still be active
    }

    #[test]
    fn test_switch_scene() {
        let mut manager = SceneManager::new();

        // Add two scenes
        let scene1 = Scene::new();
        let scene2 = Scene::new();
        manager.add_scene(scene1);
        manager.add_scene(scene2);

        // Initially, the first scene should be active
        assert_eq!(manager.current_scene_index, Some(0));

        // Switch to the second scene
        manager.switch_scene(1);
        assert_eq!(manager.current_scene_index, Some(1));
    }

    #[test]
    fn test_get_active_scene() {
        let mut manager = SceneManager::new();

        // Initially, there should be no active scene
        assert!(manager.get_active_scene().is_none());

        // Add a scene
        let scene = Scene::new();
        manager.add_scene(scene);

        // Now there should be an active scene
        assert!(manager.get_active_scene().is_some());
    }

    #[test]
    fn test_draw_active_scene() {
        let mut manager = SceneManager::new();
        let mut scene = Scene::new();

        // Add a button to the scene
        let button = Button::new(
            Point::new(0, 0),
            Size::new(10, 10),
            "Test Button",
        );
        scene.add_component(crate::common::Drawable::Button(button));

        // Add the scene to the manager
        manager.add_scene(scene);

        // Draw the active scene
        let mut display = MockDisplay::new();
        let result = manager.draw_active_scene(&mut display);

        assert!(result.is_ok());
    }

    #[test]
    fn test_update_active_scene() {
        let mut manager = SceneManager::new();
        let scene = Scene::new();

        // Add the scene to the manager
        manager.add_scene(scene);

        // Update the active scene
        manager.update_active_scene();

        // No assertion needed, just verifying it doesn't panic
    }

    #[test]
    fn test_switch_to_invalid_scene() {
        let mut manager = SceneManager::new();
        let scene = Scene::new();

        // Add one scene
        manager.add_scene(scene);

        // Try to switch to a non-existent scene
        manager.switch_scene(999);

        // The active scene should not change
        assert_eq!(manager.current_scene_index, Some(0));
    }
}
