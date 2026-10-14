use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves batched limit metrics through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct BatchedLimitMetricService {
    pub store: Rc<dyn BatchedLimitMetricStore>,
    pub validator: BatchedLimitMetricValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl BatchedLimitMetricService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("batched limit metric {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(BatchedLimitMetricStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("batched limit metric {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(BatchedLimitMetricStatus::Retired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is retired: {}", code, reason));
        Ok(())
    }

    pub fn audit(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.starts_at > cutoff {
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

pub fn new_batched_limit_metric_service(
    store: Rc<dyn BatchedLimitMetricStore>,
    clock: Rc<dyn clock::Clock>,
) -> BatchedLimitMetricService {
    BatchedLimitMetricService {
        store: store.clone(),
        validator: default_batched_limit_metric_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
