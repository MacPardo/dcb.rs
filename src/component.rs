use crate::models::{MsgContent, Timestamp};

pub trait Component {
    fn init() -> (Self, Vec<(MsgContent, Timestamp)>)
    where
        Self: Sized;

    fn on_message(self, lvt: Timestamp, msg: &MsgContent) -> (Self, Vec<(MsgContent, Timestamp)>)
    where
        Self: Sized;
}
