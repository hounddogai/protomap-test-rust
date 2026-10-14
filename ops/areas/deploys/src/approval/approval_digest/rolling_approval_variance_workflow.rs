use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves rolling approval variances through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct RollingApprovalVarianceCoordinator {
    pub store: Rc<dyn RollingApprovalVarianceStore>,
    pub validator: RollingApprovalVarianceValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl RollingApprovalVarianceCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("rolling approval variance {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(RollingApprovalVarianceStatus::Queued)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is queued", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("rolling approval variance {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(RollingApprovalVarianceStatus::Failed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is failed: {}", code, reason));
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

pub fn new_rolling_approval_variance_coordinator(
    store: Rc<dyn RollingApprovalVarianceStore>,
    clock: Rc<dyn clock::Clock>,
) -> RollingApprovalVarianceCoordinator {
    RollingApprovalVarianceCoordinator {
        store: store.clone(),
        validator: default_rolling_approval_variance_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
