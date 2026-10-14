use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves strict federation cycles through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct StrictFederationCycleWorkflow {
    pub store: Rc<dyn StrictFederationCycleStore>,
    pub validator: StrictFederationCycleValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl StrictFederationCycleWorkflow {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("strict federation cycle {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(StrictFederationCycleStatus::Approved)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is approved", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("strict federation cycle {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(StrictFederationCycleStatus::Withdrawn)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is withdrawn: {}", code, reason));
        Ok(())
    }

    pub fn audit(&mut self, cutoff: i64) -> i64 {
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

pub fn new_strict_federation_cycle_workflow(
    store: Rc<dyn StrictFederationCycleStore>,
    clock: Rc<dyn clock::Clock>,
) -> StrictFederationCycleWorkflow {
    StrictFederationCycleWorkflow {
        store: store.clone(),
        validator: default_strict_federation_cycle_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
