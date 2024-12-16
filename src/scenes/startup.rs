use embedded_graphics::{prelude::{Point, Size}, primitives::Rectangle};

use crate::{button::Button, scene::{self, scene_manager}, utils::background::Background};

pub fn build_startup() -> scene::Scene {
    let mut scene = scene::Scene::new();

    let background = Background::new(Point::new(0, 16), Size::new(320, 240 - 16));

    let btn_new_dive = Button::new(
        Point::new(52, (240 - 32) / 2),
        Size::new(100, 32),
        "New dive",
        || {
            // defmt::info!("New dive button clicked");
            unsafe {
                scene_manager::SceneManager::instance().force_unlock();
            }
            scene_manager::SceneManager::instance().lock().switch_scene(1);
        },
    );

    let btn_settings = Button::new(
        Point::new(320 - 80 - 52 - 25, (240 - 32) / 2),
        Size::new(100, 32),
        "Settings",
        || {
            // defmt::info!("Settings button clicked");
        },
    );
    scene.add_component(crate::common::Drawable::Background(background));
    scene.add_component(crate::common::Drawable::Button(btn_new_dive));
    scene.add_component(crate::common::Drawable::Button(btn_settings));
    scene.set_toggle_index(Some(0));

    scene
}
