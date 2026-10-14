use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves primary canary assignments through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct PrimaryCanaryAssignmentDesk {
    pub store: Rc<dyn PrimaryCanaryAssignmentStore>,
    pub validator: PrimaryCanaryAssignmentValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: PrimaryCanaryAssignmentEngine,
}

impl PrimaryCanaryAssignmentDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("primary canary assignment {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(PrimaryCanaryAssignmentStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("primary canary assignment {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(PrimaryCanaryAssignmentStatus::Retired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is retired: {}", code, reason));
        Ok(())
    }

    pub fn score(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("primary canary assignment {} not found", code));
        };
        let result = self.engine.price(&item);
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

pub fn new_primary_canary_assignment_desk(
    store: Rc<dyn PrimaryCanaryAssignmentStore>,
    clock: Rc<dyn clock::Clock>,
) -> PrimaryCanaryAssignmentDesk {
    PrimaryCanaryAssignmentDesk {
        store: store.clone(),
        validator: default_primary_canary_assignment_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_primary_canary_assignment_engine(),
    }
}
