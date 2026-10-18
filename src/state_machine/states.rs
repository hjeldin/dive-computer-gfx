//! Application states and the transitions between them

use crate::common::State;
use crate::scenes::{startup, dive, tissues, settings};
use crate::state_machine::{AppState, StateNode, Transition, ApplicationState};

/// Scene index of the startup scene in the scene manager
pub const STARTUP_SCENE: usize = 0;
/// Scene index of the dive scene in the scene manager
pub const DIVE_SCENE: usize = 1;
/// Scene index of the tissues scene in the scene manager
pub const TISSUES_SCENE: usize = 2;
/// Scene index of the settings scene in the scene manager
pub const SETTINGS_SCENE: usize = 3;

/// State for the startup screen
#[derive(Clone)]
pub struct StartupState {
    /// The scene index for the startup scene
    scene_index: usize,
}

impl StartupState {
    /// Create a new startup state
    pub fn new(scene_index: usize) -> Self {
        Self { scene_index }
    }
}

impl AppState for StartupState {
    fn enter(&mut self, _state: &mut State) {
        // Initialize the startup state
        // This could include setting initial values in the state
        defmt::info!("Entering startup state");
    }

    fn update(&mut self, _state: &mut State) {
        // Update the startup state
        // This could include updating timers or other state values
    }

    fn exit(&mut self, _state: &mut State) {
        // Clean up when leaving the startup state
        defmt::info!("Exiting startup state");
    }

    fn scene_index(&self) -> usize {
        self.scene_index
    }
}

/// State for the dive screen
#[derive(Clone)]
pub struct DiveState {
    /// The scene index for the dive scene
    scene_index: usize,
    /// Uptime when the dive started, in milliseconds
    started_ms: u32,
}

impl DiveState {
    /// Create a new dive state
    pub fn new(scene_index: usize) -> Self {
        Self { scene_index, started_ms: 0 }
    }
}

/// Depth in metres of a simulated dive `dive_time_ms` into it, starting at 10 m:
/// descent at 18 m/min to 25 m, bottom time until 3 min, ascent at 9 m/min to a
/// 3 min stop at 5 m, then ascent at 9 m/min to the surface.
pub fn simulated_depth(dive_time_ms: u32) -> f32 {
    const BOTTOM: f32 = 25.0;
    const BOTTOM_END_MIN: f32 = 3.0;
    const STOP: f32 = 5.0;
    const STOP_MIN: f32 = 3.0;
    const DESCENT_RATE: f32 = 18.0;
    const ASCENT_RATE: f32 = 9.0;

    let minutes = dive_time_ms as f32 / 60_000.0;
    let stop_start = BOTTOM_END_MIN + (BOTTOM - STOP) / ASCENT_RATE;
    let stop_end = stop_start + STOP_MIN;

    if minutes < BOTTOM_END_MIN {
        (10.0 + DESCENT_RATE * minutes).min(BOTTOM)
    } else if minutes < stop_start {
        BOTTOM - ASCENT_RATE * (minutes - BOTTOM_END_MIN)
    } else if minutes < stop_end {
        STOP
    } else {
        (STOP - ASCENT_RATE * (minutes - stop_end)).max(0.0)
    }
}

impl AppState for DiveState {
    fn enter(&mut self, state: &mut State) {
        // Initialize the dive state
        // This could include setting initial values in the state
        defmt::info!("Entering dive state");

        self.started_ms = state.uptime_ms;
        state.dive_time_ms = 0;
        state.depth_m = simulated_depth(0);
    }

    fn update(&mut self, state: &mut State) {
        state.dive_time_ms = state.uptime_ms.wrapping_sub(self.started_ms);
        // Stand-in for sensor data until a depth sensor is wired in
        state.depth_m = simulated_depth(state.dive_time_ms);
    }

    fn exit(&mut self, _state: &mut State) {
        // Clean up when leaving the dive state
        defmt::info!("Exiting dive state");
    }

    fn scene_index(&self) -> usize {
        self.scene_index
    }
}

/// State for the tissues screen
#[derive(Clone)]
pub struct TissuesState {
    /// The scene index for the tissues scene
    scene_index: usize,
}

impl TissuesState {
    /// Create a new tissues state
    pub fn new(scene_index: usize) -> Self {
        Self { scene_index }
    }
}

impl AppState for TissuesState {
    fn enter(&mut self, _state: &mut State) {
        // Initialize the tissues state
        defmt::info!("Entering tissues state");
    }

    fn update(&mut self, _state: &mut State) {
        // Update the tissues state
        // This could include updating tissue saturation calculations
    }

    fn exit(&mut self, _state: &mut State) {
        // Clean up when leaving the tissues state
        defmt::info!("Exiting tissues state");
    }

    fn scene_index(&self) -> usize {
        self.scene_index
    }
}

/// State for the settings screen
#[derive(Clone)]
pub struct SettingsState {
    /// The scene index for the settings scene
    scene_index: usize,
}

impl SettingsState {
    /// Create a new settings state
    pub fn new(scene_index: usize) -> Self {
        Self { scene_index }
    }
}

impl AppState for SettingsState {
    fn enter(&mut self, _state: &mut State) {
        defmt::info!("Entering settings state");
    }

    fn update(&mut self, _state: &mut State) {
        // Settings change through actions from the settings scene
    }

    fn exit(&mut self, _state: &mut State) {
        defmt::info!("Exiting settings state");
    }

    fn scene_index(&self) -> usize {
        self.scene_index
    }
}

/// Create a state node for the startup state with transitions to other states
pub fn create_startup_state() -> StateNode<ApplicationState, ApplicationState> {
    let state = ApplicationState::Startup(StartupState::new(STARTUP_SCENE));
    let mut node = StateNode::new(state);

    // Add transition to dive state below 5 m
    node.add_transition(Transition::new(
        |state| state.depth_m > 5.0,
        || ApplicationState::Dive(DiveState::new(DIVE_SCENE)),
    ));

    node
}

/// Create a state node for the dive state with transitions to other states
pub fn create_dive_state() -> StateNode<ApplicationState, ApplicationState> {
    let state = ApplicationState::Dive(DiveState::new(DIVE_SCENE));
    let mut node = StateNode::new(state);

    // Add transition to tissues state when temperature exceeds 25.0
    node.add_transition(Transition::new(
        |state| state.temperature > 25.0,
        || ApplicationState::Tissues(TissuesState::new(TISSUES_SCENE)),
    ));

    // Add transition back to startup state above 2 m
    node.add_transition(Transition::new(
        |state| state.depth_m < 2.0,
        || ApplicationState::Startup(StartupState::new(STARTUP_SCENE)),
    ));

    node
}

/// Create a state node for the tissues state with transitions to other states
pub fn create_tissues_state() -> StateNode<ApplicationState, ApplicationState> {
    let state = ApplicationState::Tissues(TissuesState::new(TISSUES_SCENE));
    let mut node = StateNode::new(state);

    // Add transition back to dive state when battery is below 50%
    node.add_transition(Transition::new(
        |state| state.battery < 128,
        || ApplicationState::Dive(DiveState::new(DIVE_SCENE)),
    ));

    node
}

/// Create a state node for the settings state with transitions to other states
pub fn create_settings_state() -> StateNode<ApplicationState, ApplicationState> {
    let state = ApplicationState::Settings(SettingsState::new(SETTINGS_SCENE));
    let mut node = StateNode::new(state);

    // A dive can start while the settings are open: switch to the dive state below 5 m
    node.add_transition(Transition::new(
        |state| state.depth_m > 5.0,
        || ApplicationState::Dive(DiveState::new(DIVE_SCENE)),
    ));

    node
}

/// Initialize the state machine with scenes and states.
/// `profile` is the dive scene's depth profile, from `dive::new_depth_profile`.
pub fn initialize_state_machine(
    scene_manager: &mut crate::scene::scene_manager::SceneManager,
    profile: &'static mut crate::components::depth_profile::DepthProfile,
) -> crate::state_machine::StateMachine {
    // Create scenes
    let startup_scene = startup::build_startup();
    let dive_scene = dive::build_dive(profile);
    let tissues_scene = tissues::build_tissues();
    let settings_scene = settings::build_settings();

    // Add scenes to scene manager, in the order the scene index constants expect
    scene_manager.add_scene(startup_scene);
    scene_manager.add_scene(dive_scene);
    scene_manager.add_scene(tissues_scene);
    scene_manager.add_scene(settings_scene);

    // Create state machine
    let mut state_machine = crate::state_machine::StateMachine::new();

    // Add states to state machine
    state_machine.add_state(create_startup_state());
    state_machine.add_state(create_dive_state());
    state_machine.add_state(create_tissues_state());
    state_machine.add_state(create_settings_state());

    state_machine
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::MAX_COMPONENTS;

    #[test]
    fn test_scenes_fit() {
        extern crate std;
        let profile = std::boxed::Box::leak(std::boxed::Box::new(dive::new_depth_profile()));
        let scenes = [
            ("startup", startup::build_startup()),
            ("dive", dive::build_dive(profile)),
            ("tissues", tissues::build_tissues()),
            ("settings", settings::build_settings()),
        ];
        for (name, scene) in &scenes {
            // A full scene drops components with only a warning
            assert!(scene.component_count() < MAX_COMPONENTS, "{} scene has {} components", name, scene.component_count());
            std::println!("{} scene: {} components", name, scene.component_count());
        }
    }

    #[test]
    fn test_simulated_dive_profile() {
        let at = |minutes: f32| simulated_depth((minutes * 60_000.0) as u32);
        assert_eq!(at(0.0), 10.0);
        assert_eq!(at(2.0), 25.0);
        assert_eq!(at(4.0), 16.0);
        assert_eq!(at(6.0), 5.0);
        assert_eq!(at(20.0), 0.0);
    }
}
