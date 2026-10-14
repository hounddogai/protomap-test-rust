use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves global dns record entries through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct GlobalDnsRecordEntryCoordinator {
    pub store: Rc<dyn GlobalDnsRecordEntryStore>,
    pub validator: GlobalDnsRecordEntryValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl GlobalDnsRecordEntryCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("global dns record entry {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(GlobalDnsRecordEntryStatus::Scheduled)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is scheduled", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("global dns record entry {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(GlobalDnsRecordEntryStatus::Cancelled)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is cancelled: {}", code, reason));
        Ok(())
    }

    pub fn sweep(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.created_at > cutoff {
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

pub fn new_global_dns_record_entry_coordinator(
    store: Rc<dyn GlobalDnsRecordEntryStore>,
    clock: Rc<dyn clock::Clock>,
) -> GlobalDnsRecordEntryCoordinator {
    GlobalDnsRecordEntryCoordinator {
        store: store.clone(),
        validator: default_global_dns_record_entry_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
