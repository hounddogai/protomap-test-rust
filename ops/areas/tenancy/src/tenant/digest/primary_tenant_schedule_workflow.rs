use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves primary tenant schedules through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct PrimaryTenantScheduleCoordinator {
    pub store: Rc<dyn PrimaryTenantScheduleStore>,
    pub validator: PrimaryTenantScheduleValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl PrimaryTenantScheduleCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("primary tenant schedule {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(PrimaryTenantScheduleStatus::Approved)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is approved", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("primary tenant schedule {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(PrimaryTenantScheduleStatus::Withdrawn)
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

pub fn new_primary_tenant_schedule_coordinator(
    store: Rc<dyn PrimaryTenantScheduleStore>,
    clock: Rc<dyn clock::Clock>,
) -> PrimaryTenantScheduleCoordinator {
    PrimaryTenantScheduleCoordinator {
        store: store.clone(),
        validator: default_primary_tenant_schedule_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
