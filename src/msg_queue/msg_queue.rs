use super::msg_queue_base::MsgQueueBase;
use crate::models::Message;
use std::sync::{Condvar, Mutex};

#[allow(dead_code)]
pub struct MsgQueue {
    queue: Mutex<MsgQueueBase>,
    cvar: Condvar,
}

impl MsgQueue {
    #[allow(dead_code)]
    pub fn new() -> MsgQueue {
        MsgQueue {
            queue: Mutex::new(MsgQueueBase::new()),
            cvar: Condvar::new(),
        }
    }

    #[allow(dead_code)]
    pub fn push(&self, msg: Message) {
        let mut queue = self.queue.lock().unwrap();
        queue.push(msg);
        if queue.size() > 0 {
            self.cvar.notify_one();
        }
    }

    #[allow(dead_code)]
    pub fn pop(&self) -> Message {
        let mut queue = self.queue.lock().unwrap();
        if queue.size() == 0 {
            let _ = self.cvar.wait(queue).unwrap();
            return self.pop();
        } else {
            return queue.pop().unwrap();
        }
    }
}
