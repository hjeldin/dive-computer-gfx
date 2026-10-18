use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*
};

use heapless::Vec;
use core::option::Option::Some;
use core::option::Option::None;
use core::result::Result;
use core::result::Result::Ok;
use core::option::Option;

use super::Scene;

/// One per scene index in `state_machine::states`.
const MAX_SCENES_COUNT: usize = 4;
/// Manages multiple scenes and handles the currently active scene.
///
/// The owner (e.g. the main loop) holds the manager. Components request scene
/// changes by returning an `Action` from a click instead of reaching a global.
pub struct SceneManager {
    scenes: Vec<Scene, MAX_SCENES_COUNT>,
    current_scene_index: Option<usize>, // None if no scene is active
}

impl SceneManager {
    /// Create a new `SceneManager`.
    pub const fn new() -> Self {
        Self {
            scenes: Vec::new(),
            current_scene_index: None,
        }
    }

    pub fn get_active_scene(&mut self) -> Option<&mut Scene> {
        self.scenes.get_mut(self.current_scene_index?)
    }

    /// Index of the active scene, if any.
    pub fn active_scene_index(&self) -> Option<usize> {
        self.current_scene_index
    }

    /// Add a new scene to the manager.
    pub fn add_scene(&mut self, scene: Scene) {
        if self.scenes.push(scene).is_err() {
            defmt::warn!("Scene manager is full, scene dropped");
            return;
        }
        if self.current_scene_index.is_none() {
            // Automatically set the first added scene as the active one.
            self.current_scene_index = Some(0);
        }
    }

    /// Switch to a different scene by index.
    /// The previous scene keeps its components, so switching back restores it.
    pub fn switch_scene(&mut self, index: usize) {
        match self.scenes.get_mut(index) {
            Some(scene) => {
                scene.activate();
                self.current_scene_index = Some(index);
            }
            None => defmt::warn!("Invalid scene index: {}", index),
        }
    }

    /// Draw the currently active scene to the display.
    pub fn draw_active_scene<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        if let Some(scene) = self.get_active_scene() {
            scene.draw(target)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::mock_display::MockDisplay;
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
        display.set_allow_out_of_bounds_drawing(true);
        display.set_allow_overdraw(true);
        let result = manager.draw_active_scene(&mut display);

        assert!(result.is_ok());
    }

    #[test]
    fn test_switch_back_keeps_components() {
        let mut manager = SceneManager::new();
        let mut scene = Scene::new();
        scene.add_component(crate::common::Drawable::Button(Button::new(
            Point::new(0, 0),
            Size::new(10, 10),
            "Test Button",
        )));
        manager.add_scene(scene);
        manager.add_scene(Scene::new());

        manager.switch_scene(1);
        manager.switch_scene(0);

        let mut display = MockDisplay::new();
        display.set_allow_out_of_bounds_drawing(true);
        display.set_allow_overdraw(true);
        manager.draw_active_scene(&mut display).unwrap();
        assert_ne!(display, MockDisplay::new());
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
