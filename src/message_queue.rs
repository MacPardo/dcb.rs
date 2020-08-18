use crate::models::Message;
use std::cmp::Ordering;
use std::cmp::Reverse;

impl Ord for Message {
    fn cmp(&self, other: &Self) -> Ordering {
        Reverse(self.exec_ts).cmp(&Reverse(other.exec_ts))
    }
}

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct MessageQueue {
    vec: Vec<Message>,
}

impl MessageQueue {
    #[allow(dead_code)]
    pub fn new() -> MessageQueue {
        MessageQueue { vec: Vec::new() }
    }

    #[allow(dead_code)]
    pub fn push(&mut self, msg: Message) {
        if let Some((index, _inverse_msg)) = self
            .vec
            .iter()
            .enumerate()
            .find(|&t| msg.is_inverse_of(&t.1))
        {
            self.vec.remove(index);
            return;
        }

        let index = match self.vec.binary_search(&msg) {
            Ok(index) => index,
            Err(index) => index,
        };
        self.vec.insert(index, msg);
    }

    #[allow(dead_code)]
    pub fn pop(&mut self) -> Option<Message> {
        self.vec.pop()
    }

    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.vec.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::models::*;
    use rand::Rng;

    fn get_msg() -> Message {
        Message {
            exec_ts: 10,
            content: MsgContent {
                path: String::default(),
                payload: String::default(),
            },
            from: 1,
            to: 2,
            id: 123,
            is_anti: false,
            sent_ts: 1,
        }
    }

    /// messages with lower exec_ts should always be greater
    #[test]
    fn messages_are_ordered_correctly() {
        let mut a = get_msg();
        a.exec_ts = 10;
        a.sent_ts = 1;
        let mut b = a.clone();
        b.exec_ts = 20;

        let mut rng = rand::thread_rng();

        for _ in 0..50 {
            b.sent_ts = rng.gen();
            a.sent_ts = rng.gen();
            assert_eq!(a.cmp(&b), Ordering::Greater);
            assert_eq!(b.cmp(&a), Ordering::Less);
            assert_eq!(a.cmp(&a), Ordering::Equal);
            assert_eq!(b.cmp(&b), Ordering::Equal);
        }
    }

    /// tests if messages are pushed correctly and if the anihilate each other when they are inverse
    #[test]
    fn push_works_correctly() {
        let mut q = MessageQueue::new();
        let mut x = get_msg();
        x.sent_ts = 5;
        let mut y = get_msg();
        y.exec_ts = 10;
        let mut z = get_msg();
        z.exec_ts = 15;
        let antix = x.get_anti().unwrap();

        q.push(x.clone());
        q.push(z.clone());
        q.push(y.clone());
        assert_eq!(q.clone().vec, vec![z.clone(), y.clone(), x.clone()]);
        q.push(antix.clone());
        assert_eq!(q.clone().vec, vec![z.clone(), y.clone()]);

        let mut q = MessageQueue::new();
        q.push(antix.clone());
        q.push(z.clone());
        q.push(y.clone());
        assert_eq!(q.clone().vec, vec![z.clone(), y.clone(), antix.clone()]);
        q.push(x.clone());
        assert_eq!(q.clone().vec, vec![z.clone(), y.clone()]);
    }

    #[test]
    fn push_maintains_correct_order() {
        let mut q = MessageQueue::new();
        let mut m = get_msg();

        let mut rng = rand::thread_rng();
        let mut aux = Vec::new();
        for _ in 0..100 {
            m.exec_ts = rng.gen();
            q.push(m.clone());
            aux.push(m.clone());
            aux.sort();
            assert_eq!(q.vec, aux);
        }
    }
}
