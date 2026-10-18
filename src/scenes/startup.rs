use embedded_graphics::prelude::{Point, Size};

use crate::{common::Action, components::button::Button, scene::{self}, state_machine::states::{DIVE_SCENE, SETTINGS_SCENE}, utils::background::Background};

use core::option::Option::None;

pub fn build_startup() -> scene::Scene {
    let mut scene = scene::Scene::new();

    let background = Background::new(Point::new(0, 16), Size::new(320, 240 - 16), None);

    let mut btn_new_dive = Button::new(
        Point::new(52, (240 - 32) / 2),
        Size::new(100, 32),
        "New dive",
    );
    btn_new_dive.set_on_click(|_button: &mut Button| Action::SwitchScene(DIVE_SCENE));

    let mut btn_settings = Button::new(
        Point::new(320 - 80 - 52 - 25, (240 - 32) / 2),
        Size::new(100, 32),
        "Settings",
    );
    btn_settings.set_on_click(|_button: &mut Button| Action::SwitchScene(SETTINGS_SCENE));
    scene.add_component(crate::common::Drawable::Background(background));
    scene.add_component(crate::common::Drawable::Button(btn_new_dive));
    scene.add_component(crate::common::Drawable::Button(btn_settings));
    scene.select_next_toggleable();

    scene
}
