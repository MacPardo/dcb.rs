use crate::models::{Checkpoint, ComponentId, Message, Timestamp};
use std::collections::{HashSet, LinkedList};

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
///     3) The save_message method is called with an antimessage
///
/// A SINGLE message is ALWAYS saved when:
///     1) The save_message method is called;
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AsyncComponentManager<State> {
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
        let mut manager = AsyncComponentManager {
            state: initial_state,
            lvt: 0,
            id: id,
            checkpoints: LinkedList::new(),
            received_messages: LinkedList::new(),
            sent_messages: LinkedList::new(),
        };
        manager.take_checkpoint();
        manager
    }

    /// This function must be called whenever the component sends or receives a message
    #[allow(dead_code)]
    pub fn save_message(&mut self, msg: Message) -> Result<(), Failure> {
        if msg.from != self.id && msg.to != self.id {
            return Err(Failure::InvalidMessage);
        }

        if msg.from == self.id {
            if let Some(last) = self.sent_messages.back() {
                if last.sent_ts > msg.sent_ts {
                    return Err(Failure::TimeViolation);
                }
            }
            self.sent_messages.push_back(msg);
        } else {
            if let Some(last) = self.received_messages.back() {
                if last.exec_ts > msg.exec_ts {
                    return Err(Failure::TimeViolation);
                }
            }
            self.received_messages.push_back(msg);
        }

        return Ok(());
    }

    /// Removes all checkpoints that were rolled back and resets the current state
    ///
    /// A checkpoint is rolled back if its timestamp is greater than or equal to rollback_ts
    ///
    /// Returns the messages that must be sent as a consequence of the rollback
    #[allow(dead_code)]
    pub fn rollback(&mut self, ts: Timestamp) -> Result<HashSet<Message>, Failure> {
        unimplemented!();
    }

    /// Deletes all checkpoints whose timestamp is not greater than ts
    /// Deletes all sent messages whose sent_ts is not greater than ts
    /// Deletes all received messages whose exec_ts is not greater than ts
    #[allow(dead_code)]
    pub fn free(&mut self, ts: Timestamp) {
        while let Some(first) = self.checkpoints.front() {
            if first.timestamp > ts {
                break;
            }
            self.checkpoints.pop_front();
        }

        while let Some(first) = self.received_messages.front() {
            if first.exec_ts > ts {
                break;
            }
            self.received_messages.pop_front();
        }

        while let Some(first) = self.sent_messages.front() {
            if first.sent_ts > ts {
                break;
            }
            self.sent_messages.pop_front();
        }
    }

    /// Saves the current state and the LVT in a Checkpoint
    #[allow(dead_code)]
    pub fn take_checkpoint(&mut self) {
        self.checkpoints.push_back(Checkpoint {
            state: self.state.clone(),
            timestamp: self.lvt,
        });
        self.lvt += 1;
    }

    /// This function must be called whenever the component's state changes
    ///
    /// Simply updates state & LVT; does not take a checkpoint
    ///
    /// Returns Err if timestamp < LVT
    #[allow(dead_code)]
    pub fn update(&mut self, state: State, lvt: Timestamp) -> Result<(), Failure> {
        if lvt < self.lvt {
            return Err(Failure::TimeViolation);
        }
        self.state = state;
        self.lvt = lvt;
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::models::*;

    fn get_manager() -> AsyncComponentManager<i32> {
        AsyncComponentManager {
            id: ComponentId {
                federate_id: 1,
                federation_id: 11,
            },
            lvt: 20,
            state: 50,
            checkpoints: LinkedList::new(),
            received_messages: LinkedList::new(),
            sent_messages: LinkedList::new(),
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
                state: initial_state,
                lvt: 1,
                id: id.clone(),
                checkpoints: checkpoints,
                sent_messages: LinkedList::new(),
                received_messages: LinkedList::new(),
            }
        );
    }

    #[test]
    fn takecheckpoint_adds_a_checkpoint_and_increments_lvt() {
        fn test(a: AsyncComponentManager<i32>) {
            let mut b = a.clone();
            b.take_checkpoint();

            let last_checkpoint = b.checkpoints.back().unwrap();
            assert_eq!(a.state, last_checkpoint.state);
            assert_eq!(a.lvt, last_checkpoint.timestamp);

            b.checkpoints.pop_back();
            b.lvt -= 1;
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
        manager.state = 20;
        manager.lvt = 10;
        let clone = manager.clone();

        let new_lvt = 11;
        let new_state = 22;
        match manager.update(new_state, new_lvt) {
            Ok(_) => (),
            Err(_) => panic!(),
        }
        assert_eq!(manager.id, clone.id);
        assert_eq!(manager.checkpoints, clone.checkpoints);
        assert_eq!(manager.sent_messages, clone.sent_messages);
        assert_eq!(manager.received_messages, clone.received_messages);

        assert_eq!(manager.state, new_state);
        assert_eq!(manager.lvt, new_lvt);
    }

    #[test]
    fn update_returns_timeviolation_if_there_is_an_attempt_to_lower_lvt_manually() {
        let original = get_manager();
        let mut manager = original.clone();

        match manager.update(123, 10) {
            Err(Failure::TimeViolation) => (),
            _ => panic!(),
        }
        assert_eq!(manager.state, original.state);
        assert_eq!(manager.lvt, original.lvt);
    }

    #[test]
    fn savemessage_appends_received_message_to_correct_list() {
        let self_id = get_id(1);
        let other_id = get_id(2);
        let mut manager = AsyncComponentManager::new(self_id.clone(), 123);
        let mut msg = get_message();
        msg.from = other_id.clone();
        msg.to = self_id.clone();
        let mut clone = manager.clone();
        manager.save_message(msg.clone()).unwrap();
        assert_ne!(manager, clone);
        clone.received_messages.push_back(msg);
        assert_eq!(manager, clone);
    }

    #[test]
    fn savemessage_appends_sent_message_to_correct_list() {
        let self_id = get_id(1);
        let other_id = get_id(2);
        let mut manager = AsyncComponentManager::new(self_id.clone(), 123);
        let mut msg = get_message();
        msg.from = self_id.clone();
        msg.to = other_id.clone();
        let mut clone = manager.clone();
        manager.save_message(msg.clone()).unwrap();
        assert_ne!(manager, clone);
        clone.sent_messages.push_back(msg);
        assert_eq!(manager, clone);
    }

    #[test]
    fn savemessage_handles_antimessages() {
        unimplemented!();
    }

    #[test]
    fn savemessage_returns_invalidmessage_if_new_message_is_neither_sent_or_received_by_self() {
        let self_id = get_id(1);
        let other_id1 = get_id(2);
        let other_id2 = get_id(3);
        let mut manager = get_manager();
        manager.id = self_id.clone();

        let mut msg = get_message();
        msg.from = other_id1;
        msg.to = other_id2;

        match manager.save_message(msg) {
            Err(Failure::InvalidMessage) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn savemessage_returns_timeviolation_if_new_sent_message_breaks_order() {
        let self_id = get_id(1);
        let other_id = get_id(2);
        let mut manager = AsyncComponentManager::new(self_id.clone(), 123);
        let mut msg1 = get_message();
        msg1.from = self_id;
        msg1.to = other_id;

        msg1.sent_ts = 10;
        msg1.exec_ts = 100;
        let mut msg2 = msg1.clone();
        msg2.sent_ts = 5;
        msg2.exec_ts = 100;

        manager.save_message(msg1).unwrap();
        match manager.save_message(msg2) {
            Err(Failure::TimeViolation) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn savemessage_return_timeviolation_if_new_received_message_breaks_order() {
        let self_id = get_id(1);
        let other_id = get_id(2);
        let mut manager = AsyncComponentManager::new(self_id.clone(), 123);
        let mut msg1 = get_message();
        msg1.from = other_id;
        msg1.to = self_id;

        msg1.exec_ts = 10;
        msg1.sent_ts = 1;
        let mut msg2 = msg1.clone();
        msg2.exec_ts = 5;

        manager.save_message(msg1).unwrap();
        match manager.save_message(msg2) {
            Err(Failure::TimeViolation) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn free_removes_correct_sent_messages() {
        let self_id = get_id(1);
        let other_id = get_id(2);
        let mut manager = AsyncComponentManager::new(self_id.clone(), 123);
        let mut msg1 = get_message();
        msg1.from = self_id;
        msg1.to = other_id;
        msg1.sent_ts = 10;
        msg1.exec_ts = 300;

        let mut msg2 = msg1.clone();
        msg2.sent_ts = 20;
        msg2.exec_ts = 200;

        let mut msg3 = msg1.clone();
        msg3.sent_ts = 30;
        msg3.exec_ts = 100;

        manager.save_message(msg1).unwrap();
        manager.save_message(msg2).unwrap();
        manager.save_message(msg3).unwrap();

        let mut clone = manager.clone();

        manager.free(20);
        assert_ne!(manager, clone);
        clone.sent_messages.pop_front();
        clone.sent_messages.pop_front();
        clone.checkpoints.pop_front();
        assert_eq!(manager, clone);
    }

    #[test]
    fn free_removes_correct_received_messages() {
        let self_id = get_id(1);
        let other_id = get_id(2);
        let mut manager = AsyncComponentManager::new(self_id.clone(), 123);
        let mut msg1 = get_message();
        msg1.from = other_id;
        msg1.to = self_id;
        msg1.sent_ts = 30;
        msg1.exec_ts = 100;

        let mut msg2 = msg1.clone();
        msg2.sent_ts = 20;
        msg2.exec_ts = 200;

        let mut msg3 = msg1.clone();
        msg3.sent_ts = 10;
        msg3.exec_ts = 300;

        manager.save_message(msg1).unwrap();
        manager.save_message(msg2).unwrap();
        manager.save_message(msg3).unwrap();

        let mut clone = manager.clone();

        manager.free(20);
        assert_ne!(manager, clone);
        clone.sent_messages.pop_front();
        clone.sent_messages.pop_front();
        clone.checkpoints.pop_front();
        assert_eq!(manager, clone);
    }

    #[test]
    fn free_removes_correct_checkpoints() {
        let mut manager = AsyncComponentManager::new(get_id(1), 123);
        manager.update(11, 10).unwrap();
        manager.take_checkpoint();
        manager.update(22, 20).unwrap();
        manager.take_checkpoint();
        manager.update(33, 30).unwrap();
        manager.take_checkpoint();

        let mut clone = manager.clone();
        manager.free(20);
        assert_ne!(manager, clone);
        clone.checkpoints.pop_front();
        clone.checkpoints.pop_front();
        clone.checkpoints.pop_front();
        assert_eq!(manager, clone);
    }

    /// The checkpoints are insufficient when there is no checkpoint whose timestamp is less than
    /// or equal to the timestamp of the rollback.
    #[test]
    fn rollback_returns_insufficientcheckpoints_if_checkpoints_are_insufficient() {
        let mut manager = get_manager();
        manager.lvt = 10;
        manager.checkpoints.clear();
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
        manager.lvt = 10;
        manager.checkpoints.clear();
        let clone = manager.clone();
        match manager.rollback(20) {
            Err(Failure::TimeViolation) => (),
            _ => panic!(),
        }
        assert_eq!(manager, clone);

        manager.checkpoints.push_back(Checkpoint {
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
