use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves dns record checkpoints through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct DnsRecordCheckpointService {
    pub store: Rc<dyn DnsRecordCheckpointStore>,
    pub validator: DnsRecordCheckpointValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl DnsRecordCheckpointService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("dns record checkpoint {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(DnsRecordCheckpointStatus::Queued)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is queued", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("dns record checkpoint {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(DnsRecordCheckpointStatus::Failed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is failed: {}", code, reason));
        Ok(())
    }

    pub fn review_stale(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.ends_at > cutoff {
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

pub fn new_dns_record_checkpoint_service(
    store: Rc<dyn DnsRecordCheckpointStore>,
    clock: Rc<dyn clock::Clock>,
) -> DnsRecordCheckpointService {
    DnsRecordCheckpointService {
        store: store.clone(),
        validator: default_dns_record_checkpoint_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
