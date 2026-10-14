use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves audit event checkpoints through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct AuditEventCheckpointDesk {
    pub store: Rc<dyn AuditEventCheckpointStore>,
    pub validator: AuditEventCheckpointValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl AuditEventCheckpointDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("audit event checkpoint {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(AuditEventCheckpointStatus::InReview)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is in review", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("audit event checkpoint {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(AuditEventCheckpointStatus::Archived)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is archived: {}", code, reason));
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

pub fn new_audit_event_checkpoint_desk(
    store: Rc<dyn AuditEventCheckpointStore>,
    clock: Rc<dyn clock::Clock>,
) -> AuditEventCheckpointDesk {
    AuditEventCheckpointDesk {
        store: store.clone(),
        validator: default_audit_event_checkpoint_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
