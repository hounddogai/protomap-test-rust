use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves flow log allocations through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct FlowLogAllocationManager {
    pub store: Rc<dyn FlowLogAllocationStore>,
    pub validator: FlowLogAllocationValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl FlowLogAllocationManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("flow log allocation {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(FlowLogAllocationStatus::Approved)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is approved", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("flow log allocation {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(FlowLogAllocationStatus::Withdrawn)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is withdrawn: {}", code, reason));
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

pub fn new_flow_log_allocation_manager(
    store: Rc<dyn FlowLogAllocationStore>,
    clock: Rc<dyn clock::Clock>,
) -> FlowLogAllocationManager {
    FlowLogAllocationManager {
        store: store.clone(),
        validator: default_flow_log_allocation_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
