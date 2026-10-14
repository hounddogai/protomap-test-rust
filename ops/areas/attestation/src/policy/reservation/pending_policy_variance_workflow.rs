use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves pending policy variances through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct PendingPolicyVarianceDesk {
    pub store: Rc<dyn PendingPolicyVarianceStore>,
    pub validator: PendingPolicyVarianceValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: PendingPolicyVarianceEngine,
}

impl PendingPolicyVarianceDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("pending policy variance {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(PendingPolicyVarianceStatus::InReview)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is in review", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("pending policy variance {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(PendingPolicyVarianceStatus::Archived)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is archived: {}", code, reason));
        Ok(())
    }

    pub fn estimate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("pending policy variance {} not found", code));
        };
        let result = self.engine.price(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
    }

    pub fn audit(&mut self, cutoff: i64) -> i64 {
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

pub fn new_pending_policy_variance_desk(
    store: Rc<dyn PendingPolicyVarianceStore>,
    clock: Rc<dyn clock::Clock>,
) -> PendingPolicyVarianceDesk {
    PendingPolicyVarianceDesk {
        store: store.clone(),
        validator: default_pending_policy_variance_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_pending_policy_variance_engine(),
    }
}
