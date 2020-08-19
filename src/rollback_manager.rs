use crate::models::{Checkpoint, ComponentId, Message, Timestamp};
use std::collections::{HashSet, LinkedList};

/// This must ONLY be used in the DCB, NOT IN THE COMPONENT.
///
/// RollbackManager is responsible for managing a async component's checkpoint & message storage.
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
pub struct RollbackManager<State> {
    state: State,
    lvt: Timestamp,
    id: ComponentId,

    // checkpoints must be in ascending timestamp order
    checkpoints: LinkedList<Checkpoint<State>>,

    // received_messages must be in ascending exec_ts order
    received_messages: LinkedList<Message>,

    // sent_messages must be in ascending sent_ts order
    sent_messages: LinkedList<Message>,
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

impl<State> RollbackManager<State>
where
    State: Clone,
{
    /// Constructor
    #[allow(dead_code)]
    pub fn new(id: ComponentId, initial_state: State) -> RollbackManager<State> {
        let mut checkpoints = LinkedList::new();
        checkpoints.push_back(Checkpoint {
            state: initial_state.clone(),
            timestamp: 0,
        });
        RollbackManager {
            state: initial_state,
            lvt: 0,
            id: id,
            checkpoints: checkpoints,
            received_messages: LinkedList::new(),
            sent_messages: LinkedList::new(),
        }
    }

    /// This function must be called whenever the component sends or receives a message
    #[allow(dead_code)]
    pub fn save_message(&mut self, msg: Message) -> Result<(), Failure> {
        if msg.from != self.id && msg.to != self.id || msg.is_anti {
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
        let mut to_be_sent: HashSet<Message> = HashSet::new();

        if ts > self.lvt {
            return Err(Failure::TimeViolation);
        }

        match self.checkpoints.front() {
            Some(first) => {
                if first.timestamp > ts {
                    return Err(Failure::InsufficientCheckpoints);
                }
            }
            None => return Err(Failure::InsufficientCheckpoints),
        }

        loop {
            match self.checkpoints.back() {
                None => panic!(),
                Some(last) => {
                    if last.timestamp > ts {
                        self.checkpoints.pop_back().unwrap();
                    } else {
                        self.lvt = last.timestamp;
                        self.state = last.state.clone();
                        break;
                    }
                }
            }
        }

        while let Some(last) = self.received_messages.back() {
            if last.exec_ts < ts {
                break;
            }
            to_be_sent.insert(self.received_messages.pop_back().unwrap());
        }

        while let Some(last) = self.sent_messages.back() {
            if last.sent_ts < ts {
                break;
            }
            let mut msg = self.sent_messages.pop_back().unwrap();
            msg.is_anti = true;
            to_be_sent.insert(msg);
        }

        return Ok(to_be_sent);
    }

    /// Deletes all checkpoints whose timestamp is not greater than ts
    ///
    /// Deletes all sent messages whose sent_ts is not greater than ts
    ///
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
        self.lvt += 1;
        self.checkpoints.push_back(Checkpoint {
            state: self.state.clone(),
            timestamp: self.lvt,
        });
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

    #[allow(dead_code)]
    pub fn get_state(&self) -> &State {
        &self.state
    }

    #[allow(dead_code)]
    pub fn get_lvt(&self) -> Timestamp {
        self.lvt
    }

    #[allow(dead_code)]
    pub fn get_sent_messages(&self) -> &LinkedList<Message> {
        &self.sent_messages
    }

    #[allow(dead_code)]
    pub fn get_received_messages(&self) -> &LinkedList<Message> {
        &self.sent_messages
    }

    #[allow(dead_code)]
    pub fn get_checkpoints(&self) -> &LinkedList<Checkpoint<State>> {
        &self.checkpoints
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::models::*;

    fn get_manager() -> RollbackManager<i32> {
        RollbackManager {
            id: 1,
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
            content: MsgContent {
                payload: String::from(""),
                path: String::from(""),
            },
            is_anti: false,
            sent_ts: 100,
            exec_ts: 200,
            from: 10,
            to: 100,
        }
    }

    #[test]
    fn new_creates_and_takes_a_checkpoint() {
        let id = 4;
        let initial_state = String::from("hello");
        let manager = RollbackManager::new(id.clone(), initial_state.clone());

        let mut checkpoints = LinkedList::new();
        checkpoints.push_back(Checkpoint {
            timestamp: 0,
            state: initial_state.clone(),
        });

        assert_eq!(
            manager,
            RollbackManager {
                state: initial_state,
                lvt: 0,
                id: id.clone(),
                checkpoints: checkpoints,
                sent_messages: LinkedList::new(),
                received_messages: LinkedList::new(),
            }
        );
    }

    #[test]
    fn takecheckpoint_increments_lvt_then_adds_a_checkpoint() {
        fn test(a: RollbackManager<i32>) {
            let mut b = a.clone();
            b.take_checkpoint();

            let last_checkpoint = b.checkpoints.back().unwrap();
            assert_eq!(a.state, last_checkpoint.state);
            assert_eq!(a.lvt + 1, last_checkpoint.timestamp);

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
        let self_id = 1;
        let other_id = 2;
        let mut manager = RollbackManager::new(self_id.clone(), 123);
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
        let self_id = 1;
        let other_id = 2;
        let mut manager = RollbackManager::new(self_id.clone(), 123);
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
    fn savemessage_returns_invalidmessage_if_new_message_is_neither_sent_or_received_by_self() {
        let self_id = 1;
        let other_id1 = 2;
        let other_id2 = 3;
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
    fn savemessage_returns_invalidmessage_if_new_message_is_anti() {
        let self_id = 1;
        let other_id = 2;
        let mut msg = get_message();
        msg.is_anti = true;
        msg.from = self_id.clone();
        msg.to = other_id.clone();
        let mut manager = get_manager();
        manager.id = self_id.clone();
        match manager.save_message(msg) {
            Err(Failure::InvalidMessage) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn savemessage_returns_timeviolation_if_new_sent_message_breaks_order() {
        let self_id = 1;
        let other_id = 2;
        let mut manager = RollbackManager::new(self_id.clone(), 123);
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
        let self_id = 1;
        let other_id = 2;
        let mut manager = RollbackManager::new(self_id.clone(), 123);
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
        let self_id = 1;
        let other_id = 2;
        let mut manager = RollbackManager::new(self_id.clone(), 123);
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
        let self_id = 1;
        let other_id = 2;
        let mut manager = RollbackManager::new(self_id.clone(), 123);
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
        let mut manager = RollbackManager::new(1, 123);
        manager.update(11, 10).unwrap();
        manager.take_checkpoint();
        manager.update(22, 20).unwrap();
        manager.take_checkpoint();
        manager.update(33, 30).unwrap();
        manager.take_checkpoint();

        println!("manager before {:#?}", manager);

        let mut clone = manager.clone();
        manager.free(21);

        println!("manager after {:#?}", manager);
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

    /// tests if the rollback function updates LVT and state correctly; removes correct checkpoints and messages; returns correct messages to be sent
    #[test]
    fn rollback_changes_values_correctly() {
        let self_id = 1;
        let other_id = 2;
        let rec1 = Message {
            content: MsgContent {
                payload: String::default(),
                path: String::default(),
            },
            from: other_id.clone(),
            to: self_id.clone(),
            sent_ts: 1,
            exec_ts: 10,
            id: 123,
            is_anti: false,
        };
        let mut rec2 = rec1.clone();
        rec2.exec_ts = 20;
        let mut rec3 = rec1.clone();
        rec3.exec_ts = 30;

        let sent1 = Message {
            content: MsgContent {
                payload: String::default(),
                path: String::default(),
            },
            from: self_id.clone(),
            to: other_id.clone(),
            sent_ts: 10,
            exec_ts: 1000,
            id: 321,
            is_anti: false,
        };
        let mut sent2 = sent1.clone();
        sent2.sent_ts = 20;
        let mut sent3 = sent1.clone();
        sent3.sent_ts = 30;

        let mut manager = RollbackManager::new(self_id.clone(), 123);
        manager.save_message(rec1.clone()).unwrap();
        manager.save_message(rec2.clone()).unwrap();
        manager.save_message(rec3.clone()).unwrap();
        manager.save_message(sent1.clone()).unwrap();
        manager.save_message(sent2.clone()).unwrap();
        manager.save_message(sent3.clone()).unwrap();

        manager.update(222, 9).unwrap();
        manager.take_checkpoint();
        manager.update(999, 19).unwrap();
        manager.take_checkpoint();
        manager.update(777, 49).unwrap();
        manager.take_checkpoint();
        manager.update(888, 200).unwrap();
        manager.take_checkpoint();

        let mut clone: RollbackManager<i32> = manager.clone();

        println!("before rollback {:#?}", manager);

        let result = manager.rollback(20).unwrap();
        assert_ne!(manager, clone);
        clone.lvt = 20;
        clone.state = 999;
        clone.checkpoints.pop_back();
        clone.checkpoints.pop_back();
        clone.sent_messages.pop_back();
        clone.sent_messages.pop_back();
        clone.received_messages.pop_back();
        clone.received_messages.pop_back();
        assert_eq!(manager, clone);

        let mut expected: HashSet<Message> = HashSet::new();
        expected.insert(rec3.clone());
        expected.insert(rec2.clone());
        let mut anti2 = sent2.clone();
        anti2.is_anti = true;
        expected.insert(anti2);
        let mut anti3 = sent3.clone();
        anti3.is_anti = true;
        expected.insert(anti3);

        println!("result {:#?}", result);
        println!("expected {:#?}", expected);

        assert_eq!(result, expected);
    }
}
