use crate::models::{Checkpoint, ComponentId, Message, Timestamp};
use std::collections::LinkedList;

/// This must ONLY be used in the DCB, NOT IN THE COMPONENT.
///
/// AsyncComponentManager is responsible for managing a async component's checkpoint & message storage.
/// Whenever there is a rollback, it informs which messages must be sent as a consequence of the
/// rollback.
///
/// A SINGLE checkpoint is ALWAYS taken when:
///     1) The constructor is called (the constructor always takes the first checkpoint);
///     2) The take_checkpoint method is called;
///
/// Checkpoints and/or messages MIGHT be freed when:
///     1) The rollback method is called;
///     2) The free_until method is called;
///
/// A SINGLE message is ALWAYS saved when:
///     1) The save_message method is called;
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AsyncComponentManager<State> {
    data: AsyncComponentData<State>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AsyncComponentData<State> {
    pub state: State,
    pub lvt: Timestamp,
    pub id: ComponentId,

    // checkpoints must be in ascending timestamp order
    pub checkpoints: LinkedList<Checkpoint<State>>,

    // received_messages must be in ascending exec_ts order
    pub received_messages: LinkedList<Message>,

    // sent_messages must be in ascending sent_ts order
    pub sent_messages: LinkedList<Message>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Failure {
    /// Occurs when an operation is inconsistent with the current LVT or breaks the ordering of messages
    TimeViolation,

    /// This occurs when there is a rollback to a timestamp T, but there is no longer any checkpoint
    /// whose timestamp is less than or equal to T
    InsufficientCheckpoints,

    /// A message is invalid when the component is neither the destiny nor the destination
    /// or if it is an antimessage
    InvalidMessage,
}

impl<State> AsyncComponentManager<State>
where
    State: Clone,
{
    /// Constructor
    #[allow(dead_code)]
    pub fn new(id: ComponentId, initial_state: State) -> AsyncComponentManager<State> {
        let data = AsyncComponentData {
            state: initial_state,
            lvt: 0,
            id: id,
            checkpoints: LinkedList::new(),
            received_messages: LinkedList::new(),
            sent_messages: LinkedList::new(),
        };
        let mut manager = AsyncComponentManager { data: data };
        manager.take_checkpoint();
        manager
    }

    /// This function must be called whenever the component sends or receives a message
    #[allow(dead_code)]
    pub fn save_message(&mut self, message: Message) -> Result<(), Failure> {
        if message.from != self.data.id && message.to != self.data.id || message.is_anti {
            return Err(Failure::InvalidMessage);
        }

        return Ok(());
    }

    /// Removes all checkpoints that were rolled back and resets the current state
    ///
    /// A checkpoint is rolled back if its timestamp is greater than or equal to rollback_ts
    ///
    /// Returns the messages that must be sent as a consequence of the rollback
    #[allow(dead_code)]
    pub fn rollback(&mut self, rollback_ts: Timestamp) -> Result<LinkedList<Message>, Failure> {
        unimplemented!();
    }

    /// Deletes all messages and checkpoints whose timestamp is not greater than the passed argument
    #[allow(dead_code)]
    pub fn free(&mut self, timestamp: Timestamp) {
        unimplemented!();
    }

    /// Saves the current state and the LVT in a Checkpoint
    #[allow(dead_code)]
    pub fn take_checkpoint(&mut self) {
        self.data.checkpoints.push_back(Checkpoint {
            state: self.data.state.clone(),
            timestamp: self.data.lvt,
        });
        self.data.lvt += 1;
    }

    /// This function must be called whenever the component's state changes
    ///
    /// Simply updates state & LVT; does not take a checkpoint
    ///
    /// Returns Err if timestamp < LVT
    #[allow(dead_code)]
    pub fn update(&mut self, state: State, lvt: Timestamp) -> Result<(), Failure> {
        if lvt < self.data.lvt {
            return Err(Failure::TimeViolation);
        }
        self.data.state = state;
        self.data.lvt = lvt;
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::models::*;

    fn get_manager() -> AsyncComponentManager<i32> {
        AsyncComponentManager {
            data: AsyncComponentData {
                id: ComponentId {
                    federate_id: 1,
                    federation_id: 11,
                },
                lvt: 20,
                state: 50,
                checkpoints: LinkedList::new(),
                received_messages: LinkedList::new(),
                sent_messages: LinkedList::new(),
            },
        }
    }

    fn get_message() -> Message {
        Message {
            id: 10,
            content: String::from("lkadjsfkl"),
            is_anti: false,
            sent_ts: 100,
            exec_ts: 200,
            from: ComponentId {
                federate_id: 10,
                federation_id: 20,
            },
            to: ComponentId {
                federate_id: 100,
                federation_id: 200,
            },
        }
    }

    fn get_id(x: u32) -> ComponentId {
        ComponentId {
            federate_id: x,
            federation_id: x,
        }
    }

    #[test]
    fn new_creates_and_takes_a_checkpoint() {
        let id = ComponentId {
            federate_id: 4,
            federation_id: 5,
        };
        let initial_state = String::from("hello");
        let manager = AsyncComponentManager::new(id.clone(), initial_state.clone());

        let mut checkpoints = LinkedList::new();
        checkpoints.push_back(Checkpoint {
            timestamp: 0,
            state: initial_state.clone(),
        });

        assert_eq!(
            manager,
            AsyncComponentManager {
                data: AsyncComponentData {
                    state: initial_state,
                    lvt: 1,
                    id: id.clone(),
                    checkpoints: checkpoints,
                    sent_messages: LinkedList::new(),
                    received_messages: LinkedList::new(),
                }
            }
        );
    }

    #[test]
    fn takecheckpoint_adds_a_checkpoint_and_increments_lvt() {
        fn test(a: AsyncComponentManager<i32>) {
            let mut b = a.clone();
            b.take_checkpoint();

            let last_checkpoint = b.data.checkpoints.back().unwrap();
            assert_eq!(a.data.state, last_checkpoint.state);
            assert_eq!(a.data.lvt, last_checkpoint.timestamp);

            b.data.checkpoints.pop_back();
            b.data.lvt -= 1;
            assert_eq!(a, b);
        }

        let mut a = get_manager();

        for _ in 0..10 {
            test(a.clone());
            a.take_checkpoint();
        }
    }

    #[test]
    fn update_changes_fields_correctly() {
        let mut manager = get_manager();
        manager.data.state = 20;
        manager.data.lvt = 10;
        let clone = manager.clone();

        let new_lvt = 11;
        let new_state = 22;
        match manager.update(new_state, new_lvt) {
            Ok(_) => (),
            Err(_) => panic!(),
        }
        assert_eq!(manager.data.id, clone.data.id);
        assert_eq!(manager.data.checkpoints, clone.data.checkpoints);
        assert_eq!(manager.data.sent_messages, clone.data.sent_messages);
        assert_eq!(manager.data.received_messages, clone.data.received_messages);

        assert_eq!(manager.data.state, new_state);
        assert_eq!(manager.data.lvt, new_lvt);
    }

    #[test]
    fn update_returns_timeviolation_if_there_is_an_attempt_to_lower_lvt_manually() {
        let original = get_manager();
        let mut manager = original.clone();

        match manager.update(123, 10) {
            Err(Failure::TimeViolation) => (),
            _ => panic!(),
        }
        assert_eq!(manager.data.state, original.data.state);
        assert_eq!(manager.data.lvt, original.data.lvt);
    }

    #[test]
    fn savemessage_appends_received_message_to_correct_list() {
        unimplemented!();
    }

    #[test]
    fn savemessage_appends_sent_message_to_correct_list() {
        unimplemented!();
    }

    #[test]
    fn savemessage_returns_invalidmessage_if_new_message_is_anti() {
        let self_id = get_id(1);
        let other_id = get_id(2);
        let mut msg = get_message();
        msg.is_anti = true;
        msg.from = self_id.clone();
        msg.to = other_id.clone();
        let mut manager = get_manager();
        manager.data.id = self_id.clone();
        match manager.save_message(msg) {
            Err(Failure::InvalidMessage) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn savemessage_returns_invalidmessage_if_new_message_is_neither_sent_or_received_by_self() {
        let self_id = get_id(1);
        let other_id1 = get_id(2);
        let other_id2 = get_id(3);
        let mut manager = get_manager();
        manager.data.id = self_id.clone();

        let mut msg = get_message();
        msg.from = other_id1;
        msg.to = other_id2;

        match manager.save_message(msg) {
            Err(Failure::InvalidMessage) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn savemessage_returns_timeviolation_if_new_sent_message_breaks_order() {}

    #[test]
    fn savemessage_return_timeviolation_if_new_received_message_breaks_order() {
        unimplemented!();
    }

    #[test]
    fn free_removes_correct_sent_messages() {
        unimplemented!();
    }

    #[test]
    fn free_removes_correct_received_messages() {
        unimplemented!();
    }

    #[test]
    fn free_removes_correct_checkpoints() {
        unimplemented!();
    }

    /// The checkpoints are insufficient when there is no checkpoint whose timestamp is less than
    /// or equal to the timestamp of the rollback.
    #[test]
    fn rollback_returns_insufficientcheckpoints_if_checkpoints_are_insufficient() {
        let mut manager = get_manager();
        manager.data.lvt = 10;
        manager.data.checkpoints.clear();
        let clone = manager.clone();
        match manager.rollback(5) {
            Err(Failure::InsufficientCheckpoints) => (),
            _ => panic!(),
        };
        assert_eq!(manager, clone);
    }

    #[test]
    fn rollback_retuns_timeviolation_if_there_is_an_attempt_to_rollback_the_future() {
        let mut manager = get_manager();
        manager.data.lvt = 10;
        manager.data.checkpoints.clear();
        let clone = manager.clone();
        match manager.rollback(20) {
            Err(Failure::TimeViolation) => (),
            _ => panic!(),
        }
        assert_eq!(manager, clone);

        manager.data.checkpoints.push_back(Checkpoint {
            state: 123,
            timestamp: 5,
        });
        let clone = manager.clone();
        match manager.rollback(20) {
            Err(Failure::TimeViolation) => (),
            _ => panic!(),
        }
        assert_eq!(manager, clone);
    }

    #[test]
    fn rollback_updates_state_and_lvt_correctly() {
        unimplemented!();
    }

    #[test]
    fn rollback_removes_correct_messages_and_checkpoints() {
        unimplemented!();
    }

    #[test]
    fn rollback_returns_the_messages_that_must_be_sent_by_the_component() {
        unimplemented!();
    }
}
