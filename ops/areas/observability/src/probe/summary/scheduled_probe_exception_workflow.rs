use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves scheduled probe exceptions through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct ScheduledProbeExceptionCoordinator {
    pub store: Rc<dyn ScheduledProbeExceptionStore>,
    pub validator: ScheduledProbeExceptionValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl ScheduledProbeExceptionCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("scheduled probe exception {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(ScheduledProbeExceptionStatus::InReview)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is in review", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("scheduled probe exception {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(ScheduledProbeExceptionStatus::Archived)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is archived: {}", code, reason));
        Ok(())
    }

    pub fn sweep(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.expires_at > cutoff {
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

pub fn new_scheduled_probe_exception_coordinator(
    store: Rc<dyn ScheduledProbeExceptionStore>,
    clock: Rc<dyn clock::Clock>,
) -> ScheduledProbeExceptionCoordinator {
    ScheduledProbeExceptionCoordinator {
        store: store.clone(),
        validator: default_scheduled_probe_exception_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
