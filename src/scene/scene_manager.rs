use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*
};

use heapless::Vec;
use once_cell::sync::OnceCell;
use spin::Mutex;

use super::Scene;

const MAX_SCENES_COUNT: usize = 32;
/// Manages multiple scenes and handles the currently active scene.
pub struct SceneManager {
    scenes: Vec<Scene, MAX_SCENES_COUNT>,
    current_scene_index: Option<usize>, // None if no scene is active
}

impl SceneManager {
    /// Create a new `SceneManager`.
    pub fn new() -> Self {
        Self {
            scenes: Vec::new(),
            current_scene_index: None,
        }
    }

    pub fn instance() -> &'static Mutex<SceneManager> {
        static INSTANCE: OnceCell<Mutex<SceneManager>> = OnceCell::new();
        INSTANCE.get_or_init(|| Mutex::new(SceneManager::new()))
    }

    pub fn get_active_scene(&mut self) -> Option<&mut Scene> {
        if let Some(index) = self.current_scene_index {
            self.scenes.get_mut(index)
        } else {
            None
        }
    }

    /// Add a new scene to the manager.
    pub fn add_scene(&mut self, scene: Scene) {
        self.scenes.push(scene);
        if self.current_scene_index.is_none() {
            // Automatically set the first added scene as the active one.
            self.current_scene_index = Some(0);
        }
    }

    /// Switch to a different scene by index.
    pub fn switch_scene(&mut self, index: usize) {
        if index < self.scenes.len() {
            self.current_scene_index = Some(index);
        } else {
            // defmt::error!("Invalid scene index: {}", index);
        }
    }

    /// Update the currently active scene.
    pub fn update_active_scene(&mut self) {
        if let Some(index) = self.current_scene_index {
            if let Some(scene) = self.scenes.get_mut(index) {
                // Perform updates specific to the active scene (e.g., toggling or clicking items).
                // scene.toggle_next_toggleable(); // Example of scene-specific updates
            }
        }
    }

    /// Draw the currently active scene to the display.
    pub fn draw_active_scene<D: DrawTarget<Color = Rgb565>>(&self, target: &mut D) -> Result<(), D::Error> {
        if let Some(index) = self.current_scene_index {
            if let Some(scene) = self.scenes.get(index) {
                scene.draw(target)?; // Delegate drawing to the active scene
            }
        }
        Ok(())
    }
}
