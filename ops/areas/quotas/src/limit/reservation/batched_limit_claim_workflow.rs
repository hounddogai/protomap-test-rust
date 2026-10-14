use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves batched limit claims through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct BatchedLimitClaimCoordinator {
    pub store: Rc<dyn BatchedLimitClaimStore>,
    pub validator: BatchedLimitClaimValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: BatchedLimitClaimEngine,
}

impl BatchedLimitClaimCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("batched limit claim {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(BatchedLimitClaimStatus::Granted)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is granted", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("batched limit claim {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(BatchedLimitClaimStatus::Expired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is expired: {}", code, reason));
        Ok(())
    }

    pub fn evaluate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("batched limit claim {} not found", code));
        };
        let result = self.engine.run(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
    }

    pub fn scan(&mut self, cutoff: i64) -> i64 {
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

pub fn new_batched_limit_claim_coordinator(
    store: Rc<dyn BatchedLimitClaimStore>,
    clock: Rc<dyn clock::Clock>,
) -> BatchedLimitClaimCoordinator {
    BatchedLimitClaimCoordinator {
        store: store.clone(),
        validator: default_batched_limit_claim_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_batched_limit_claim_engine(),
    }
}
