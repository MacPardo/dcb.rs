use crate::models::{Checkpoint, ComponentId, Message, Timestamp};
use std::collections::LinkedList;

/// This must ONLY be used in the DCB, NOT IN THE COMPONENT.
///
/// AsyncComponentData is responsible for managing a async component's checkopint & message storage.
/// Whenever there is a rollback, it informs which messages must be sent as a consequence of the rollback.
///
/// A SINGLE checkpoint is ALWAYS taken when:
///     1. The constructor is called (the constructor always takes the first checkpoint)
///     2. The take_checkpoint method is called
///
/// Checkpoints and/or messages MIGHT be freed when:
///     1. The rollback_to method is called
///     2. The free_until method is called
///
/// A SINGLE message is ALWAYS saved when:
///     1. The save_message method is called
#[derive(Debug, Eq, PartialEq)]
pub struct AsyncComponentData<AppState> {
    state: AppState,
    lvt: Timestamp,
    id: ComponentId,

    // checkpoints, received_messages & sent_messages must always be in ascending timestamp order
    checkpoints: LinkedList<Checkpoint<AppState>>,
    received_messages: LinkedList<Message>,
    sent_messages: LinkedList<Message>,
}

impl<AppState> AsyncComponentData<AppState>
where
    AppState: Clone,
{
    /// Constructor
    pub fn new(id: ComponentId, initial_state: AppState) -> AsyncComponentData<AppState> {
        unimplemented!();
    }

    /// This function must be called whenever the component sends or receives a message
    pub fn save_message(&mut self, message: Message) {
        unimplemented!();
    }

    /// Removes all checkpoints that were rolled back and resets the current state
    /// Returns the messages that must be sent as a consequence of the rollback
    pub fn rollback_to(&mut self, rollback_ts: Timestamp) -> LinkedList<Message> {
        unimplemented!();
    }

    /// Deletes all messages and checkopints whose timestamp is not greater than the passed argument
    pub fn free_up_to(&mut self, timestamp: Timestamp) {
        unimplemented!();
    }

    /// Saves the current state and the LVT in a Checkpoint
    /// It's the DCB's responsibility to decide when to take a checkpoint, not the component's
    pub fn take_checkpoint(&mut self) {
        unimplemented!();
    }

    /// This function must be called whenever the component's state changes
    /// Simply updates state & LVT; does not take a checkpoint
    pub fn update(&mut self, state: AppState, timestamp: Timestamp) {
        unimplemented!();
    }

    /// Returns the current state
    /// This must be the single source of truth for the state; the component must not store its own state
    pub fn get_state(&self) -> AppState {
        self.state.clone()
    }

    /// Returns the current LVT
    /// This must be the single source of truth for the LVT; the component must not store its own LVT
    pub fn get_lvt(&self) -> Timestamp {
        self.lvt
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::models::*;

    #[test]
    fn new_creates_and_takes_a_checkpoint() {
        let id = ComponentId {
            federate_id: 4,
            federation_id: 5,
        };
        let initial_state = String::from("hello");
        let data = AsyncComponentData::new(id.clone(), initial_state.clone());

        let mut checkpoints = LinkedList::new();
        checkpoints.push_back(Checkpoint {
            timestamp: 0,
            state: initial_state.clone(),
        });

        assert_eq!(
            data,
            AsyncComponentData {
                state: initial_state,
                lvt: 1,
                id: id.clone(),
                checkpoints: checkpoints,
                sent_messages: LinkedList::new(),
                received_messages: LinkedList::new()
            }
        );
    }

    #[test]
    fn take_checkpoint_works() {}
}
