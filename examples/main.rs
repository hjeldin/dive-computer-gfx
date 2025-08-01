// #![no_std]
// Note: This example requires the simulator feature to be enabled.
// Run with: cargo run --example main --features "std simulator"
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::sdl2::Keycode;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

#[cfg(feature = "std")]
extern crate std;

use stm_graphics::{components, State};
use stm_graphics::status;
use stm_graphics::utils;
use stm_graphics::scene;
use stm_graphics::scenes;
use stm_graphics::state_machine;
use stm_graphics::state_machine::examples;

fn main() -> Result<(), core::convert::Infallible> {
    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(320, 240));

    let output_settings = OutputSettingsBuilder::new().scale(2).build();

    let mut status_bar = status::StatusBar::new(22, 32);
    let mut battery_indicator = utils::battery::BatteryIndicator::new(0xAA);

    let mut scene_manager = unsafe { &mut scene::scene_manager::INSTANCE };

    // Initialize the state machine with scenes
    let mut state_machine = examples::initialize_state_machine(scene_manager);

    let mut wnd = Window::new("Dive computer", &output_settings);
    let _ = status_bar.draw(&mut display);
    let _ = battery_indicator.draw(&mut display);
    let mut quit = false;

    // The state is now managed by the state machine
    let s = state_machine.state_mut();
    s.battery = 0x00;
    s.pressure = 0.22;
    s.temperature = 25.0;

    loop {
        if quit {
            return Ok(());
        }
        // if(s.battery - 1 > 0) {
        //     s.battery -= 1;
        // } else {
        //     s.battery = 0xFF;
        // }

        // Get the current state from the state machine
        let current_state = state_machine.state();

        // Update the battery indicator with the current state
        battery_indicator.set_level(current_state.battery);
        let _ = battery_indicator.draw(&mut display);

        // Draw the active scene
        let _ = scene_manager.draw_active_scene(&mut display);
        wnd.update(&display);

        // Update the state machine, which will handle state transitions
        state_machine.update(scene_manager);

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

                Keycode::U => {
                    // defmt::info!("U key pressed");
                    // Update pressure in the state machine's state
                    let s = state_machine.state_mut();
                    s.pressure += 0.1;
                }

                Keycode::H => {
                    // Increase heading
                    let s = state_machine.state_mut();
                    s.heading = (s.heading + 10.0) % 360.0;
                }

                Keycode::J => {
                    // Decrease heading
                    let s = state_machine.state_mut();
                    s.heading = (s.heading - 10.0 + 360.0) % 360.0;
                }

                Keycode::K => {
                    // Increase set course
                    let s = state_machine.state_mut();
                    s.set_course = (s.set_course + 10.0) % 360.0;
                }

                Keycode::L => {
                    // Decrease set course
                    let s = state_machine.state_mut();
                    s.set_course = (s.set_course - 10.0 + 360.0) % 360.0;
                }

                _ => {
                    // defmt::info!("Key pressed: {:?}", key);
                }
            },
            _ => {}
        });
    }
}
