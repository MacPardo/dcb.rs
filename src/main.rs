mod models;
mod rollback_manager;

use models::ComponentId;
use rollback_manager::RollbackManager;

fn main() {
    for i in 0..10 {
        println!("{}", i);
    }
}

#[cfg(test)]
mod test {}
