use embedded_graphics::prelude::{Point, Size};

use crate::{components::button::Button, scene, utils::background::Background};

use core::option::Option::None;

pub fn build_tissues() -> scene::Scene {
    let mut scene = scene::Scene::new();
    let background = Background::new(Point::new(0, 16), Size::new(320, 240 - 16), None);
    let btn_new_dive = Button::new(
        Point::new(110, (240 - 32) / 2),
        Size::new(100, 32),
        "HALLO"
    );

    scene.add_component(crate::common::Drawable::Background(background));
    scene.add_component(crate::common::Drawable::Button(btn_new_dive));
    scene.select_next_toggleable();

    scene
}
