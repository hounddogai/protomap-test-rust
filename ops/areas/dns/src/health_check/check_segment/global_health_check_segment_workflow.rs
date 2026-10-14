use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves global health check segments through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct GlobalHealthCheckSegmentWorkflow {
    pub store: Rc<dyn GlobalHealthCheckSegmentStore>,
    pub validator: GlobalHealthCheckSegmentValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl GlobalHealthCheckSegmentWorkflow {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("global health check segment {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(GlobalHealthCheckSegmentStatus::Scheduled)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is scheduled", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("global health check segment {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(GlobalHealthCheckSegmentStatus::Cancelled)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is cancelled: {}", code, reason));
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

pub fn new_global_health_check_segment_workflow(
    store: Rc<dyn GlobalHealthCheckSegmentStore>,
    clock: Rc<dyn clock::Clock>,
) -> GlobalHealthCheckSegmentWorkflow {
    GlobalHealthCheckSegmentWorkflow {
        store: store.clone(),
        validator: default_global_health_check_segment_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
