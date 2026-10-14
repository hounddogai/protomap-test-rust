use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves fallback dns record allocations through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct FallbackDnsRecordAllocationDesk {
    pub store: Rc<dyn FallbackDnsRecordAllocationStore>,
    pub validator: FallbackDnsRecordAllocationValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl FallbackDnsRecordAllocationDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback dns record allocation {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(FallbackDnsRecordAllocationStatus::Pending)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is pending", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback dns record allocation {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(FallbackDnsRecordAllocationStatus::Closed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is closed: {}", code, reason));
        Ok(())
    }

    pub fn review_stale(&mut self, cutoff: i64) -> i64 {
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

pub fn new_fallback_dns_record_allocation_desk(
    store: Rc<dyn FallbackDnsRecordAllocationStore>,
    clock: Rc<dyn clock::Clock>,
) -> FallbackDnsRecordAllocationDesk {
    FallbackDnsRecordAllocationDesk {
        store: store.clone(),
        validator: default_fallback_dns_record_allocation_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
