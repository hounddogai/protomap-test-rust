use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves scheduled redaction assignments through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct ScheduledRedactionAssignmentDesk {
    pub store: Rc<dyn ScheduledRedactionAssignmentStore>,
    pub validator: ScheduledRedactionAssignmentValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl ScheduledRedactionAssignmentDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("scheduled redaction assignment {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(ScheduledRedactionAssignmentStatus::Granted)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is granted", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("scheduled redaction assignment {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(ScheduledRedactionAssignmentStatus::Expired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is expired: {}", code, reason));
        Ok(())
    }

    pub fn scan(&mut self, cutoff: i64) -> i64 {
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

pub fn new_scheduled_redaction_assignment_desk(
    store: Rc<dyn ScheduledRedactionAssignmentStore>,
    clock: Rc<dyn clock::Clock>,
) -> ScheduledRedactionAssignmentDesk {
    ScheduledRedactionAssignmentDesk {
        store: store.clone(),
        validator: default_scheduled_redaction_assignment_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
