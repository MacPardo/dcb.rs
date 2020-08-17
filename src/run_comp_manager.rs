use crate::message_queue::MessageQueue;
use crate::messenger::Messenger;
use crate::models::{ComponentCfg, ComponentId, Message};
use rand::Rng;
use std::sync::mpsc::Receiver;
use std::thread;

#[allow(dead_code)]
pub fn run_comp_manager(_cfg: ComponentCfg, _messenger: Messenger, _receiver: Receiver<Message>) {
    let mut message_queue = MessageQueue::new();

    thread::spawn(move || {
        for msg in _receiver {
            println!("${:?} got a message: {:#?}", _cfg.id, msg);
            message_queue.push(msg);
        }
    });

    let mut rng = rand::thread_rng();
    loop {
        thread::sleep(std::time::Duration::from_millis(100));
        let id = rng.gen_range(1, 4);
        _messenger
            .send(Message {
                content: String::from("lalala"),
                sent_ts: 10,
                exec_ts: 20,
                is_anti: false,
                id: 1,
                from: _cfg.id,
                to: ComponentId {
                    federate_id: id,
                    federation_id: 1,
                },
            })
            .unwrap();
    }
}
