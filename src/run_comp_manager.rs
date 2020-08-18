use crate::gateway::Gateway;
use crate::messenger::Messenger;
use crate::models::{ComponentCfg, Message};
use crate::rollback_manager::RollbackManager;
use crate::sync_msg_queue::SyncMsgQueue;
use std::sync::{mpsc::Receiver, Arc};

#[allow(dead_code)]
pub fn run_comp_manager<State: Clone>(
    config: ComponentCfg,
    initial_state: State,
    gateway: impl Gateway<State>,
    messenger: Messenger,
    receiver: Receiver<Message>,
) {
    let queue = Arc::new(SyncMsgQueue::new());

    let queue_clone = queue.clone();
    std::thread::spawn(move || {
        for msg in receiver {
            queue_clone.push(msg);
        }
    });

    for msg in gateway.init() {
        messenger.send(msg).unwrap();
    }

    let mut rollback_manager = RollbackManager::new(config.id, initial_state.clone());
    let mut state = initial_state;

    loop {
        let msg = queue.pop();

        let violates_lcc = msg.exec_ts < rollback_manager.get_lvt();
        if violates_lcc {
            let msgs = rollback_manager.rollback(msg.exec_ts).unwrap();
            for msg in msgs {
                messenger.send(msg).unwrap();
            }
        }

        rollback_manager.save_message(msg.clone()).unwrap();

        let (new_state, msgs) = gateway.on_message(state, rollback_manager.get_lvt(), &msg);
        state = new_state;

        for msg in msgs {
            rollback_manager.save_message(msg.clone()).unwrap();
            messenger.send(msg).unwrap();
        }
    }
}
