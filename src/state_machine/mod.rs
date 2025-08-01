use core::marker::PhantomData;
use heapless::Vec;
use crate::common::State;
use crate::scene::scene_manager::SceneManager;

pub mod examples;

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

    /// The target state to transition to
    target: PhantomData<S>,

    /// The factory function to create the target state
    create_target: fn() -> S,
}

impl<S> Transition<S> {
    /// Create a new transition
    pub fn new(condition: fn(&State) -> bool, create_target: fn() -> S) -> Self {
        Self {
            condition,
            target: PhantomData,
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
        let _ = self.transitions.push(transition);
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
    Startup(examples::StartupState),
    Dive(examples::DiveState),
    Tissues(examples::TissuesState),
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
    /// The current state node
    current_state: Option<StateNode<ApplicationState, ApplicationState>>,

    /// All available states
    states: Vec<StateNode<ApplicationState, ApplicationState>, MAX_STATES>,

    /// The application state
    app_state: State,
}

impl StateMachine {
    /// Create a new state machine
    pub fn new() -> Self {
        Self {
            current_state: None,
            states: Vec::new(),
            app_state: State::new(),
        }
    }

    /// Add a state to the state machine
    pub fn add_state(&mut self, state_node: StateNode<ApplicationState, ApplicationState>) {
        let _ = self.states.push(state_node);

        // If this is the first state, make it the current state
        if self.current_state.is_none() && !self.states.is_empty() {
            self.current_state = Some(self.states.swap_remove(0));

            // Initialize the state
            if let Some(ref mut current) = self.current_state {
                current.state_mut().enter(&mut self.app_state);
            }
        }
    }

    /// Update the state machine
    pub fn update(&mut self, scene_manager: &mut SceneManager) {
        if let Some(ref mut current) = self.current_state {
            // Update the current state
            current.state_mut().update(&mut self.app_state);

            // Check for transitions
            let next_state = current.check_transitions(&self.app_state);

            if let Some(next) = next_state {
                // Exit the current state
                current.state_mut().exit(&mut self.app_state);

                // Create a new state node for the next state
                let mut next_node = StateNode::new(next);

                // Find the transitions for the next state
                for state in &mut self.states {
                    // In a real implementation, we would need a better way to match states
                    // This is a simplified approach
                    match (state.state_mut(), next_node.state_mut()) {
                        (ApplicationState::Startup(_), ApplicationState::Startup(_)) |
                        (ApplicationState::Dive(_), ApplicationState::Dive(_)) |
                        (ApplicationState::Tissues(_), ApplicationState::Tissues(_)) => {
                            // Copy transitions from the matching state
                            for transition in &state.transitions {
                                next_node.add_transition(Transition::new(
                                    transition.condition,
                                    transition.create_target,
                                ));
                            }
                            break;
                        }
                        _ => {}
                    }
                }

                // Enter the next state
                next_node.state_mut().enter(&mut self.app_state);

                // Switch to the scene for the next state
                scene_manager.switch_scene(next_node.scene_index());

                // Set the next state as the current state
                self.current_state = Some(next_node);
            }
        }

        // Update all components in the active scene with the current state
        if let Some(scene) = scene_manager.get_active_scene() {
            scene.update(&self.app_state);
        }
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
