use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves rolling binding adjustments through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct RollingBindingAdjustmentCoordinator {
    pub store: Rc<dyn RollingBindingAdjustmentStore>,
    pub validator: RollingBindingAdjustmentValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl RollingBindingAdjustmentCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("rolling binding adjustment {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(RollingBindingAdjustmentStatus::Approved)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is approved", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("rolling binding adjustment {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(RollingBindingAdjustmentStatus::Withdrawn)
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

pub fn new_rolling_binding_adjustment_coordinator(
    store: Rc<dyn RollingBindingAdjustmentStore>,
    clock: Rc<dyn clock::Clock>,
) -> RollingBindingAdjustmentCoordinator {
    RollingBindingAdjustmentCoordinator {
        store: store.clone(),
        validator: default_rolling_binding_adjustment_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
