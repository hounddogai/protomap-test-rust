use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves fallback binding milestones through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct FallbackBindingMilestoneDesk {
    pub store: Rc<dyn FallbackBindingMilestoneStore>,
    pub validator: FallbackBindingMilestoneValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: FallbackBindingMilestoneEngine,
}

impl FallbackBindingMilestoneDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback binding milestone {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(FallbackBindingMilestoneStatus::InReview)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is in review", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback binding milestone {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(FallbackBindingMilestoneStatus::Archived)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is archived: {}", code, reason));
        Ok(())
    }

    pub fn quote(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("fallback binding milestone {} not found", code));
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

pub fn new_fallback_binding_milestone_desk(
    store: Rc<dyn FallbackBindingMilestoneStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> FallbackBindingMilestoneDesk {
    FallbackBindingMilestoneDesk {
        store: store.clone(),
        validator: default_fallback_binding_milestone_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_fallback_binding_milestone_engine(flags),
    }
}
