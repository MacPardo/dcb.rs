use crate::models::{Message, Timestamp};

pub trait Gateway<State> {
    fn init(&self) -> (State, Vec<Message>);

    fn on_message(&self, state: State, lvt: Timestamp, message: &Message) -> (State, Vec<Message>);
}
