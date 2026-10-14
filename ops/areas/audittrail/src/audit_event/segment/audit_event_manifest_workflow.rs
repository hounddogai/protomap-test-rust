use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves audit event manifests through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct AuditEventManifestDesk {
    pub store: Rc<dyn AuditEventManifestStore>,
    pub validator: AuditEventManifestValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl AuditEventManifestDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("audit event manifest {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(AuditEventManifestStatus::Approved)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is approved", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("audit event manifest {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(AuditEventManifestStatus::Withdrawn)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is withdrawn: {}", code, reason));
        Ok(())
    }

    pub fn scan(&mut self, cutoff: i64) -> i64 {
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

pub fn new_audit_event_manifest_desk(
    store: Rc<dyn AuditEventManifestStore>,
    clock: Rc<dyn clock::Clock>,
) -> AuditEventManifestDesk {
    AuditEventManifestDesk {
        store: store.clone(),
        validator: default_audit_event_manifest_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
