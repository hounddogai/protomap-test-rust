use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves retention estimates through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct RetentionEstimateWorkflow {
    pub store: Rc<dyn RetentionEstimateStore>,
    pub validator: RetentionEstimateValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl RetentionEstimateWorkflow {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("retention estimate {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(RetentionEstimateStatus::Granted)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is granted", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("retention estimate {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(RetentionEstimateStatus::Expired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is expired: {}", code, reason));
        Ok(())
    }

    pub fn review_stale(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.updated_at > cutoff {
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

pub fn new_retention_estimate_workflow(
    store: Rc<dyn RetentionEstimateStore>,
    clock: Rc<dyn clock::Clock>,
) -> RetentionEstimateWorkflow {
    RetentionEstimateWorkflow {
        store: store.clone(),
        validator: default_retention_estimate_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
