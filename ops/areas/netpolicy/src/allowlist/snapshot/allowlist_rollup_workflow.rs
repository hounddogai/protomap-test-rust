use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves allowlist rollups through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct AllowlistRollupCoordinator {
    pub store: Rc<dyn AllowlistRollupStore>,
    pub validator: AllowlistRollupValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl AllowlistRollupCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("allowlist rollup {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(AllowlistRollupStatus::Queued)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is queued", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("allowlist rollup {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(AllowlistRollupStatus::Failed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is failed: {}", code, reason));
        Ok(())
    }

    pub fn sweep(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
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

pub fn new_allowlist_rollup_coordinator(
    store: Rc<dyn AllowlistRollupStore>,
    clock: Rc<dyn clock::Clock>,
) -> AllowlistRollupCoordinator {
    AllowlistRollupCoordinator {
        store: store.clone(),
        validator: default_allowlist_rollup_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
