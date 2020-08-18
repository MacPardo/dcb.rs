use crate::models::{Message, Timestamp};

pub trait Gateway<State> {
    fn init(&self) -> Vec<Message>;

    fn on_message(&self, state: State, lvt: Timestamp, message: &Message) -> (State, Vec<Message>);
}
