use crate::gateway::Gateway;
use crate::messenger::Messenger;
use crate::models::ComponentCfg;
use crate::rollback_manager::RollbackManager;
use crate::sync_msg_queue::SyncMsgQueue;
use std::sync::Arc;

#[allow(dead_code)]
pub fn consume_msg_queue<State: Clone>(
    config: ComponentCfg,
    gateway: impl Gateway<State>,
    messenger: Messenger,
    queue: Arc<SyncMsgQueue>,
) {
    let (initial_state, initial_messages) = gateway.init();
    for msg in initial_messages {
        messenger.send(msg).unwrap();
    }

    let mut rollback_manager = RollbackManager::new(config.id, initial_state.clone());
    let mut state = initial_state;

    loop {
        let msg = queue.pop();

        let violates_lcc = msg.exec_ts < rollback_manager.lvt();
        if violates_lcc {
            let msgs = rollback_manager.rollback(msg.exec_ts).unwrap();
            for msg in msgs {
                messenger.send(msg).unwrap();
            }
        }

        rollback_manager.save_message(msg.clone()).unwrap();

        let (new_state, msgs) = gateway.on_message(state, rollback_manager.lvt(), &msg);
        state = new_state;

        for msg in msgs {
            rollback_manager.save_message(msg.clone()).unwrap();
            messenger.send(msg).unwrap();
        }
    }
}
