use crate::models::{MsgContent, Timestamp};

pub trait Component {
    fn init() -> Vec<(MsgContent, Timestamp)>;

    fn on_message(self, lvt: Timestamp, msg: &MsgContent) -> (Self, Vec<(MsgContent, Timestamp)>)
    where
        Self: Sized;
}
