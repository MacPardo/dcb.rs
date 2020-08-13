use crate::models::Message;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

#[allow(dead_code)]
struct MessageQueue {
    heap: BinaryHeap<Reverse<Message>>,
    anti_messages: HashSet<Message>,
}

impl MessageQueue {}
