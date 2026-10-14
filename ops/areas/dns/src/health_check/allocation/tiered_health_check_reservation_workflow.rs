use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves tiered health check reservations through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct TieredHealthCheckReservationManager {
    pub store: Rc<dyn TieredHealthCheckReservationStore>,
    pub validator: TieredHealthCheckReservationValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl TieredHealthCheckReservationManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("tiered health check reservation {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(TieredHealthCheckReservationStatus::InReview)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is in review", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("tiered health check reservation {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(TieredHealthCheckReservationStatus::Archived)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is archived: {}", code, reason));
        Ok(())
    }

    pub fn review_stale(&mut self, cutoff: i64) -> i64 {
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

pub fn new_tiered_health_check_reservation_manager(
    store: Rc<dyn TieredHealthCheckReservationStore>,
    clock: Rc<dyn clock::Clock>,
) -> TieredHealthCheckReservationManager {
    TieredHealthCheckReservationManager {
        store: store.clone(),
        validator: default_tiered_health_check_reservation_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
