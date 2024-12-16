use embedded_graphics::prelude::{Point, Size};

use crate::{components::button::Button, scene, utils::background::Background};

pub fn build_tissues() -> scene::Scene {
    let mut scene = scene::Scene::new();
    let background = Background::new(Point::new(0, 16), Size::new(320, 240 - 16));
    let btn_new_dive = Button::new(
        Point::new(110, (240 - 32) / 2),
        Size::new(100, 32),
        "HALLO",
        || {
            // defmt::info!("New dive button clicked");
        },
    );

    scene.add_component(crate::common::Drawable::Background(background));
    scene.add_component(crate::common::Drawable::Button(btn_new_dive));
    scene.set_toggle_index(Some(0));

    scene
}
