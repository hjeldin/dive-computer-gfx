use embedded_graphics::prelude::{Point, Size};

use crate::{components::button::Button, scene::{self}, utils::background::Background};

use core::option::Option::Some;
use core::option::Option::None;

pub fn build_startup() -> scene::Scene {
    let mut scene = scene::Scene::new();

    let background = Background::new(Point::new(0, 16), Size::new(320, 240 - 16), None);

    let mut btn_new_dive = Button::new(
        Point::new(52, (240 - 32) / 2),
        Size::new(100, 32),
        "New dive",
    );
    btn_new_dive.set_on_click(|button: &mut Button| {
        // let s = crate::scene_manager::SceneManager::instance();
        let scene_manager = unsafe { &mut scene::scene_manager::INSTANCE };
        scene_manager.switch_scene(1);
    });

    let mut btn_settings = Button::new(
        Point::new(320 - 80 - 52 - 25, (240 - 32) / 2),
        Size::new(100, 32),
        "Settings",
    );
    scene.add_component(crate::common::Drawable::Background(background));
    scene.add_component(crate::common::Drawable::Button(btn_new_dive));
    scene.add_component(crate::common::Drawable::Button(btn_settings));
    scene.set_toggle_index(Some(0));

    scene
}
