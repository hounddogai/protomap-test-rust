use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves global action item claims through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct GlobalActionItemClaimCoordinator {
    pub store: Rc<dyn GlobalActionItemClaimStore>,
    pub validator: GlobalActionItemClaimValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl GlobalActionItemClaimCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("global action item claim {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(GlobalActionItemClaimStatus::Warming)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is warming", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("global action item claim {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(GlobalActionItemClaimStatus::Stopped)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is stopped: {}", code, reason));
        Ok(())
    }

    pub fn review_stale(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.created_at > cutoff {
                continue;
            }
            let problems = self.validator.validate(&item);
            if (problems.len() as i64) == 0 {
                continue;
            }
            self.note(&format!("{} needs attention: {}", item.code, problems.join(", ")));
            changed += 1;
        }
        changed
    }
}

pub fn new_global_action_item_claim_coordinator(
    store: Rc<dyn GlobalActionItemClaimStore>,
    clock: Rc<dyn clock::Clock>,
) -> GlobalActionItemClaimCoordinator {
    GlobalActionItemClaimCoordinator {
        store: store.clone(),
        validator: default_global_action_item_claim_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
