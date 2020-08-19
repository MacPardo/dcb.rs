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
        let received = queue.pop();

        let violates_lcc = received.content.exec_ts < rollback_manager.lvt();
        if violates_lcc {
            let msgs = rollback_manager.rollback(received.content.exec_ts).unwrap();
            for msg in msgs {
                messenger.send(msg).unwrap();
            }
        }

        rollback_manager.save_message(received.clone()).unwrap();

        let (new_state, msgs) = gateway.on_message(state, &received);
        rollback_manager
            .update(new_state.clone(), received.content.exec_ts)
            .unwrap();
        state = new_state;

        for msg in msgs {
            rollback_manager.save_message(msg.clone()).unwrap();
            messenger.send(msg).unwrap();
        }
    }
}
