use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves batched tenant milestones through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct BatchedTenantMilestoneManager {
    pub store: Rc<dyn BatchedTenantMilestoneStore>,
    pub validator: BatchedTenantMilestoneValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl BatchedTenantMilestoneManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("batched tenant milestone {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(BatchedTenantMilestoneStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("batched tenant milestone {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(BatchedTenantMilestoneStatus::Retired)
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

pub fn new_batched_tenant_milestone_manager(
    store: Rc<dyn BatchedTenantMilestoneStore>,
    clock: Rc<dyn clock::Clock>,
) -> BatchedTenantMilestoneManager {
    BatchedTenantMilestoneManager {
        store: store.clone(),
        validator: default_batched_tenant_milestone_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
