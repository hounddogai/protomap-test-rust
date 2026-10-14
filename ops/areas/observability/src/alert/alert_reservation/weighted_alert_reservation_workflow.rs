use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves weighted alert reservations through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct WeightedAlertReservationDesk {
    pub store: Rc<dyn WeightedAlertReservationStore>,
    pub validator: WeightedAlertReservationValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl WeightedAlertReservationDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("weighted alert reservation {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(WeightedAlertReservationStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("weighted alert reservation {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(WeightedAlertReservationStatus::Retired)
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

pub fn new_weighted_alert_reservation_desk(
    store: Rc<dyn WeightedAlertReservationStore>,
    clock: Rc<dyn clock::Clock>,
) -> WeightedAlertReservationDesk {
    WeightedAlertReservationDesk {
        store: store.clone(),
        validator: default_weighted_alert_reservation_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
