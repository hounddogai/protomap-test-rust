use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves legacy rollback policies through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct LegacyRollbackPolicyCoordinator {
    pub store: Rc<dyn LegacyRollbackPolicyStore>,
    pub validator: LegacyRollbackPolicyValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl LegacyRollbackPolicyCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("legacy rollback policy {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(LegacyRollbackPolicyStatus::InReview)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is in review", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("legacy rollback policy {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(LegacyRollbackPolicyStatus::Archived)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is archived: {}", code, reason));
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

pub fn new_legacy_rollback_policy_coordinator(
    store: Rc<dyn LegacyRollbackPolicyStore>,
    clock: Rc<dyn clock::Clock>,
) -> LegacyRollbackPolicyCoordinator {
    LegacyRollbackPolicyCoordinator {
        store: store.clone(),
        validator: default_legacy_rollback_policy_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
