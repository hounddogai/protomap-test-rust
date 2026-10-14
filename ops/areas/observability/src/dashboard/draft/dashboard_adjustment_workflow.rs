use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves dashboard adjustments through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct DashboardAdjustmentWorkflow {
    pub store: Rc<dyn DashboardAdjustmentStore>,
    pub validator: DashboardAdjustmentValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl DashboardAdjustmentWorkflow {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("dashboard adjustment {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(DashboardAdjustmentStatus::Queued)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is queued", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("dashboard adjustment {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(DashboardAdjustmentStatus::Failed)
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
            if item.ends_at > cutoff {
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

pub fn new_dashboard_adjustment_workflow(
    store: Rc<dyn DashboardAdjustmentStore>,
    clock: Rc<dyn clock::Clock>,
) -> DashboardAdjustmentWorkflow {
    DashboardAdjustmentWorkflow {
        store: store.clone(),
        validator: default_dashboard_adjustment_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
