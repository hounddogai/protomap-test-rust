use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves flow log mappings through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct FlowLogMappingManager {
    pub store: Rc<dyn FlowLogMappingStore>,
    pub validator: FlowLogMappingValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl FlowLogMappingManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("flow log mapping {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(FlowLogMappingStatus::Approved)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is approved", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("flow log mapping {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(FlowLogMappingStatus::Withdrawn)
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

pub fn new_flow_log_mapping_manager(
    store: Rc<dyn FlowLogMappingStore>,
    clock: Rc<dyn clock::Clock>,
) -> FlowLogMappingManager {
    FlowLogMappingManager {
        store: store.clone(),
        validator: default_flow_log_mapping_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
