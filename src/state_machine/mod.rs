use heapless::Vec;
use crate::common::{Action, State};
use crate::scene::scene_manager::SceneManager;

pub mod states;

/// Represents a state in the state machine
pub trait AppState {
    /// Initialize the state
    fn enter(&mut self, state: &mut State);

    /// Update the state
    fn update(&mut self, state: &mut State);

    /// Clean up when leaving the state
    fn exit(&mut self, state: &mut State);

    /// Get the scene index associated with this state
    fn scene_index(&self) -> usize;
}

/// Represents a transition between states
pub struct Transition<S> {
    /// The condition that triggers this transition
    condition: fn(&State) -> bool,

    /// The factory function to create the target state
    create_target: fn() -> S,
}

impl<S> Transition<S> {
    /// Create a new transition
    pub fn new(condition: fn(&State) -> bool, create_target: fn() -> S) -> Self {
        Self {
            condition,
            create_target,
        }
    }

    /// Check if the transition should be triggered
    pub fn should_transition(&self, state: &State) -> bool {
        (self.condition)(state)
    }

    /// Create the target state
    pub fn create_target(&self) -> S {
        (self.create_target)()
    }
}

/// The maximum number of transitions from a single state
const MAX_TRANSITIONS: usize = 8;

/// A state with its possible transitions
pub struct StateNode<S, T> {
    /// The current state
    state: S,

    /// Possible transitions from this state
    transitions: Vec<Transition<T>, MAX_TRANSITIONS>,
}

impl<S, T> StateNode<S, T> 
where 
    S: AppState,
{
    /// Create a new state node
    pub fn new(state: S) -> Self {
        Self {
            state,
            transitions: Vec::new(),
        }
    }

    /// Add a transition to this state
    pub fn add_transition(&mut self, transition: Transition<T>) {
        if self.transitions.push(transition).is_err() {
            defmt::warn!("State node is full, transition dropped");
        }
    }

    /// Check if any transition should be triggered
    pub fn check_transitions(&self, state: &State) -> Option<T> {
        for transition in &self.transitions {
            if transition.should_transition(state) {
                return Some(transition.create_target());
            }
        }
        None
    }

    /// Get a mutable reference to the state
    pub fn state_mut(&mut self) -> &mut S {
        &mut self.state
    }

    /// Get the scene index for this state
    pub fn scene_index(&self) -> usize {
        self.state.scene_index()
    }
}

/// The maximum number of states in the state machine
const MAX_STATES: usize = 8;

/// Enum to represent all possible application states
#[derive(Clone)]
pub enum ApplicationState {
    Startup(states::StartupState),
    Dive(states::DiveState),
    Tissues(states::TissuesState),
}

impl AppState for ApplicationState {
    fn enter(&mut self, state: &mut State) {
        match self {
            ApplicationState::Startup(s) => s.enter(state),
            ApplicationState::Dive(s) => s.enter(state),
            ApplicationState::Tissues(s) => s.enter(state),
        }
    }

    fn update(&mut self, state: &mut State) {
        match self {
            ApplicationState::Startup(s) => s.update(state),
            ApplicationState::Dive(s) => s.update(state),
            ApplicationState::Tissues(s) => s.update(state),
        }
    }

    fn exit(&mut self, state: &mut State) {
        match self {
            ApplicationState::Startup(s) => s.exit(state),
            ApplicationState::Dive(s) => s.exit(state),
            ApplicationState::Tissues(s) => s.exit(state),
        }
    }

    fn scene_index(&self) -> usize {
        match self {
            ApplicationState::Startup(s) => s.scene_index(),
            ApplicationState::Dive(s) => s.scene_index(),
            ApplicationState::Tissues(s) => s.scene_index(),
        }
    }
}

/// A state machine that manages application states and transitions
pub struct StateMachine {
    /// All available states. Nodes stay here for the machine's lifetime.
    states: Vec<StateNode<ApplicationState, ApplicationState>, MAX_STATES>,

    /// Index into `states` of the current state
    current: Option<usize>,

    /// The application state
    app_state: State,
}

impl StateMachine {
    /// Create a new state machine
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
            current: None,
            app_state: State::new(),
        }
    }

    /// Add a state to the state machine.
    /// The first state added becomes the current state.
    pub fn add_state(&mut self, state_node: StateNode<ApplicationState, ApplicationState>) {
        if self.states.push(state_node).is_err() {
            defmt::warn!("State machine is full, state dropped");
            return;
        }
        if self.current.is_none() {
            self.current = Some(0);
            self.states[0].state_mut().enter(&mut self.app_state);
        }
    }

    /// Update the state machine
    pub fn update(&mut self, scene_manager: &mut SceneManager) {
        if let Some(current) = self.current {
            self.states[current].state_mut().update(&mut self.app_state);
            if let Some(next) = self.states[current].check_transitions(&self.app_state) {
                self.enter_scene(next.scene_index(), scene_manager);
            }
        }

        // Update all components in the active scene with the current state
        if let Some(scene) = scene_manager.get_active_scene() {
            scene.update(&self.app_state);
        }
    }

    /// Apply an action requested by a component, such as a button click.
    pub fn handle_action(&mut self, action: Action, scene_manager: &mut SceneManager) {
        match action {
            Action::None => {}
            Action::SwitchScene(index) => self.enter_scene(index, scene_manager),
        }
    }

    /// Exit the current state, then enter the state that owns `scene_index`
    /// and show its scene.
    fn enter_scene(&mut self, scene_index: usize, scene_manager: &mut SceneManager) {
        let Some(next) = self.states.iter().position(|node| node.scene_index() == scene_index) else {
            defmt::warn!("No state for scene index: {}", scene_index);
            return;
        };
        if let Some(current) = self.current {
            self.states[current].state_mut().exit(&mut self.app_state);
        }
        self.states[next].state_mut().enter(&mut self.app_state);
        scene_manager.switch_scene(scene_index);
        self.current = Some(next);
    }

    /// Get the scene index of the current state
    pub fn current_scene_index(&self) -> Option<usize> {
        self.current.map(|index| self.states[index].scene_index())
    }

    /// Get a reference to the application state
    pub fn state(&self) -> &State {
        &self.app_state
    }

    /// Get a mutable reference to the application state
    pub fn state_mut(&mut self) -> &mut State {
        &mut self.app_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Scene;

    fn setup() -> (StateMachine, SceneManager) {
        let mut scene_manager = SceneManager::new();
        for _ in 0..3 {
            scene_manager.add_scene(Scene::new());
        }
        let mut state_machine = StateMachine::new();
        state_machine.add_state(states::create_startup_state());
        state_machine.add_state(states::create_dive_state());
        state_machine.add_state(states::create_tissues_state());
        (state_machine, scene_manager)
    }

    #[test]
    fn test_first_state_is_current() {
        let (state_machine, _) = setup();
        assert_eq!(state_machine.current_scene_index(), Some(states::STARTUP_SCENE));
    }

    #[test]
    fn test_round_trip_keeps_transitions() {
        let (mut state_machine, mut scene_manager) = setup();

        // Startup -> Dive on pressure
        state_machine.state_mut().pressure = 6.0;
        state_machine.update(&mut scene_manager);
        assert_eq!(state_machine.current_scene_index(), Some(states::DIVE_SCENE));
        assert_eq!(scene_manager.active_scene_index(), Some(states::DIVE_SCENE));

        // Dive -> Startup once the simulated dive surfaces
        state_machine.state_mut().uptime_ms = 20 * 60_000;
        state_machine.update(&mut scene_manager);
        assert_eq!(state_machine.state().dive_time_ms, 20 * 60_000);
        assert_eq!(state_machine.current_scene_index(), Some(states::STARTUP_SCENE));

        // Startup still has its transition after leaving and coming back
        state_machine.state_mut().pressure = 6.0;
        state_machine.update(&mut scene_manager);
        assert_eq!(state_machine.current_scene_index(), Some(states::DIVE_SCENE));
    }

    #[test]
    fn test_handle_action_switches_state_and_scene() {
        let (mut state_machine, mut scene_manager) = setup();

        state_machine.handle_action(Action::SwitchScene(states::TISSUES_SCENE), &mut scene_manager);
        assert_eq!(state_machine.current_scene_index(), Some(states::TISSUES_SCENE));
        assert_eq!(scene_manager.active_scene_index(), Some(states::TISSUES_SCENE));

        state_machine.handle_action(Action::None, &mut scene_manager);
        assert_eq!(state_machine.current_scene_index(), Some(states::TISSUES_SCENE));
    }
}
