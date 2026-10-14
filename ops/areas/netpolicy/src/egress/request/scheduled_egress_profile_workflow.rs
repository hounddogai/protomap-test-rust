use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves scheduled egress profiles through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct ScheduledEgressProfileDesk {
    pub store: Rc<dyn ScheduledEgressProfileStore>,
    pub validator: ScheduledEgressProfileValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl ScheduledEgressProfileDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("scheduled egress profile {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(ScheduledEgressProfileStatus::Queued)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is queued", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("scheduled egress profile {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(ScheduledEgressProfileStatus::Failed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is failed: {}", code, reason));
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

pub fn new_scheduled_egress_profile_desk(
    store: Rc<dyn ScheduledEgressProfileStore>,
    clock: Rc<dyn clock::Clock>,
) -> ScheduledEgressProfileDesk {
    ScheduledEgressProfileDesk {
        store: store.clone(),
        validator: default_scheduled_egress_profile_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
