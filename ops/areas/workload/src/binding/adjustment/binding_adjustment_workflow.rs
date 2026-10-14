use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves binding adjustments through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct BindingAdjustmentCoordinator {
    pub store: Rc<dyn BindingAdjustmentStore>,
    pub validator: BindingAdjustmentValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: BindingAdjustmentEngine,
}

impl BindingAdjustmentCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("binding adjustment {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(BindingAdjustmentStatus::Queued)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is queued", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("binding adjustment {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(BindingAdjustmentStatus::Failed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is failed: {}", code, reason));
        Ok(())
    }

    pub fn estimate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("binding adjustment {} not found", code));
        };
        let result = self.engine.run(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
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

pub fn new_binding_adjustment_coordinator(
    store: Rc<dyn BindingAdjustmentStore>,
    clock: Rc<dyn clock::Clock>,
) -> BindingAdjustmentCoordinator {
    BindingAdjustmentCoordinator {
        store: store.clone(),
        validator: default_binding_adjustment_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_binding_adjustment_engine(),
    }
}
