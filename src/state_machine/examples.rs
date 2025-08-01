//! Example implementations of the AppState trait for different scenes

use crate::common::State;
use crate::scenes::{startup, dive, tissues};
use crate::state_machine::{AppState, StateNode, Transition, ApplicationState};

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
}

impl DiveState {
    /// Create a new dive state
    pub fn new(scene_index: usize) -> Self {
        Self { scene_index }
    }
}

impl AppState for DiveState {
    fn enter(&mut self, state: &mut State) {
        // Initialize the dive state
        // This could include setting initial values in the state
        defmt::info!("Entering dive state");

        // Example: Set initial pressure for dive
        state.pressure = 10.0;
    }

    fn update(&mut self, state: &mut State) {
        // Update the dive state
        // This could include updating depth, NDT, etc. based on sensor data

        // Example: Simulate pressure changes
        state.pressure += 0.1;
        if state.pressure > 30.0 {
            state.pressure = 10.0;
        }
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

/// Create a state node for the startup state with transitions to other states
pub fn create_startup_state(scene_index: usize) -> StateNode<ApplicationState, ApplicationState> {
    let state = ApplicationState::Startup(StartupState::new(scene_index));
    let mut node = StateNode::new(state);

    // Add transition to dive state when pressure exceeds 5.0
    node.add_transition(Transition::new(
        |state| state.pressure > 5.0,
        || ApplicationState::Dive(DiveState::new(1)), // Assuming dive scene is at index 1
    ));

    node
}

/// Create a state node for the dive state with transitions to other states
pub fn create_dive_state(scene_index: usize) -> StateNode<ApplicationState, ApplicationState> {
    let state = ApplicationState::Dive(DiveState::new(scene_index));
    let mut node = StateNode::new(state);

    // Add transition to tissues state when temperature exceeds 25.0
    node.add_transition(Transition::new(
        |state| state.temperature > 25.0,
        || ApplicationState::Tissues(TissuesState::new(2)), // Assuming tissues scene is at index 2
    ));

    // Add transition back to startup state when pressure is below 2.0
    node.add_transition(Transition::new(
        |state| state.pressure < 2.0,
        || ApplicationState::Startup(StartupState::new(0)), // Assuming startup scene is at index 0
    ));

    node
}

/// Create a state node for the tissues state with transitions to other states
pub fn create_tissues_state(scene_index: usize) -> StateNode<ApplicationState, ApplicationState> {
    let state = ApplicationState::Tissues(TissuesState::new(scene_index));
    let mut node = StateNode::new(state);

    // Add transition back to dive state when battery is below 50%
    node.add_transition(Transition::new(
        |state| state.battery < 128,
        || ApplicationState::Dive(DiveState::new(1)), // Assuming dive scene is at index 1
    ));

    node
}

/// Initialize the state machine with scenes and states
pub fn initialize_state_machine(scene_manager: &mut crate::scene::scene_manager::SceneManager) -> crate::state_machine::StateMachine {
    // Create scenes
    let startup_scene = startup::build_startup();
    let dive_scene = dive::build_dive();
    let tissues_scene = tissues::build_tissues();

    // Add scenes to scene manager
    scene_manager.add_scene(startup_scene);
    scene_manager.add_scene(dive_scene);
    scene_manager.add_scene(tissues_scene);

    // Create state machine
    let mut state_machine = crate::state_machine::StateMachine::new();

    // Add states to state machine
    state_machine.add_state(create_startup_state(0));
    state_machine.add_state(create_dive_state(1));
    state_machine.add_state(create_tissues_state(2));

    state_machine
}
