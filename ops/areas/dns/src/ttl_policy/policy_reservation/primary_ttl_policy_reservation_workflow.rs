use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves primary ttl policy reservations through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct PrimaryTtlPolicyReservationDesk {
    pub store: Rc<dyn PrimaryTtlPolicyReservationStore>,
    pub validator: PrimaryTtlPolicyReservationValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: PrimaryTtlPolicyReservationEngine,
}

impl PrimaryTtlPolicyReservationDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("primary ttl policy reservation {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(PrimaryTtlPolicyReservationStatus::Queued)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is queued", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("primary ttl policy reservation {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(PrimaryTtlPolicyReservationStatus::Failed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is failed: {}", code, reason));
        Ok(())
    }

    pub fn evaluate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("primary ttl policy reservation {} not found", code));
        };
        let result = self.engine.compute(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
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

pub fn new_primary_ttl_policy_reservation_desk(
    store: Rc<dyn PrimaryTtlPolicyReservationStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> PrimaryTtlPolicyReservationDesk {
    PrimaryTtlPolicyReservationDesk {
        store: store.clone(),
        validator: default_primary_ttl_policy_reservation_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_primary_ttl_policy_reservation_engine(flags),
    }
}
