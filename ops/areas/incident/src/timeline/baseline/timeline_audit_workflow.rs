use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves timeline audits through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct TimelineAuditManager {
    pub store: Rc<dyn TimelineAuditStore>,
    pub validator: TimelineAuditValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl TimelineAuditManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("timeline audit {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(TimelineAuditStatus::Pending)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is pending", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("timeline audit {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(TimelineAuditStatus::Closed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is closed: {}", code, reason));
        Ok(())
    }

    pub fn sweep(&mut self, cutoff: i64) -> i64 {
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

pub fn new_timeline_audit_manager(
    store: Rc<dyn TimelineAuditStore>,
    clock: Rc<dyn clock::Clock>,
) -> TimelineAuditManager {
    TimelineAuditManager {
        store: store.clone(),
        validator: default_timeline_audit_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
