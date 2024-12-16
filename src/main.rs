#![no_std]
use button::Button;
use common::Toggleable;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics_simulator::sdl2::Keycode;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use heapless::Vec;

mod button;
mod common;
mod scene;
mod scenes;
mod status;
mod utils;

fn main() -> Result<(), core::convert::Infallible> {
    let selected: Option<&mut dyn Toggleable>;

    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(320, 240));

    let output_settings = OutputSettingsBuilder::new().scale(2).build();

    let status_bar = status::StatusBar::new(22, 32);
    let battery_indicator = utils::battery::BatteryIndicator::new(0xAA);

    let mut scene_manager = scene::scene_manager::SceneManager::instance().lock();
    

    scene_manager.add_scene(scenes::startup::build_startup());
    scene_manager.add_scene(scenes::dive::build_dive());
    

    let mut wnd = Window::new("Dive computer", &output_settings);
    let _ = status_bar.draw(&mut display);
    let _ = battery_indicator.draw(&mut display);
    let mut quit = false;
    loop {
        if quit {
            return Ok(());
        }
        let _ = scene_manager.draw_active_scene(&mut display);
        wnd.update(&display);
        wnd.events().for_each(|event| match event {
            SimulatorEvent::Quit => {
                // defmt::info!("Quitting");
                quit = true;
                return;
            }
            SimulatorEvent::KeyDown {
                keycode: key,
                keymod: _,
                repeat: _,
            } => match key {
                Keycode::Q => {
                    // defmt::info!("Quitting");
                    quit = true;
                    return;
                }

                Keycode::RIGHT => {
                    // defmt::info!("Right key pressed");
                    scene_manager.get_active_scene().unwrap().select_next_toggleable();
                }

                Keycode::LEFT => {
                    // defmt::info!("Left key pressed");
                    scene_manager.get_active_scene().unwrap().select_prev_toggleable();
                }

                Keycode::RETURN => {
                    // defmt::info!("Return key pressed");
                    scene_manager.get_active_scene().unwrap().click_currently_toggled();
                }

                _ => {
                    // defmt::info!("Key pressed: {:?}", key);
                }
            },
            _ => {}
        });
    }
}
