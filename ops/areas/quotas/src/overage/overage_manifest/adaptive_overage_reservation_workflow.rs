use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves adaptive overage reservations through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct AdaptiveOverageReservationService {
    pub store: Rc<dyn AdaptiveOverageReservationStore>,
    pub validator: AdaptiveOverageReservationValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl AdaptiveOverageReservationService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("adaptive overage reservation {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(AdaptiveOverageReservationStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("adaptive overage reservation {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(AdaptiveOverageReservationStatus::Retired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is retired: {}", code, reason));
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

pub fn new_adaptive_overage_reservation_service(
    store: Rc<dyn AdaptiveOverageReservationStore>,
    clock: Rc<dyn clock::Clock>,
) -> AdaptiveOverageReservationService {
    AdaptiveOverageReservationService {
        store: store.clone(),
        validator: default_adaptive_overage_reservation_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
