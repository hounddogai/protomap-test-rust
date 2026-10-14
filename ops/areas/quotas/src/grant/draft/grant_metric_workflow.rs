use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves grant metrics through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct GrantMetricCoordinator {
    pub store: Rc<dyn GrantMetricStore>,
    pub validator: GrantMetricValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl GrantMetricCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("grant metric {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(GrantMetricStatus::Queued)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is queued", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("grant metric {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(GrantMetricStatus::Failed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is failed: {}", code, reason));
        Ok(())
    }

    pub fn scan(&mut self, cutoff: i64) -> i64 {
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

pub fn new_grant_metric_coordinator(
    store: Rc<dyn GrantMetricStore>,
    clock: Rc<dyn clock::Clock>,
) -> GrantMetricCoordinator {
    GrantMetricCoordinator {
        store: store.clone(),
        validator: default_grant_metric_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
