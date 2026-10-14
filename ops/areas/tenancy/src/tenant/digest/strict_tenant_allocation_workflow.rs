use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves strict tenant allocations through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct StrictTenantAllocationManager {
    pub store: Rc<dyn StrictTenantAllocationStore>,
    pub validator: StrictTenantAllocationValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl StrictTenantAllocationManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("strict tenant allocation {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(StrictTenantAllocationStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("strict tenant allocation {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(StrictTenantAllocationStatus::Retired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is retired: {}", code, reason));
        Ok(())
    }

    pub fn scan(&mut self, cutoff: i64) -> i64 {
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

pub fn new_strict_tenant_allocation_manager(
    store: Rc<dyn StrictTenantAllocationStore>,
    clock: Rc<dyn clock::Clock>,
) -> StrictTenantAllocationManager {
    StrictTenantAllocationManager {
        store: store.clone(),
        validator: default_strict_tenant_allocation_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
