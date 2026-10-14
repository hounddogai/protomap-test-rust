use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves fallback dashboard reservations through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct FallbackDashboardReservationCoordinator {
    pub store: Rc<dyn FallbackDashboardReservationStore>,
    pub validator: FallbackDashboardReservationValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl FallbackDashboardReservationCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback dashboard reservation {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(FallbackDashboardReservationStatus::Scheduled)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is scheduled", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback dashboard reservation {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(FallbackDashboardReservationStatus::Cancelled)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is cancelled: {}", code, reason));
        Ok(())
    }

    pub fn audit(&mut self, cutoff: i64) -> i64 {
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

pub fn new_fallback_dashboard_reservation_coordinator(
    store: Rc<dyn FallbackDashboardReservationStore>,
    clock: Rc<dyn clock::Clock>,
) -> FallbackDashboardReservationCoordinator {
    FallbackDashboardReservationCoordinator {
        store: store.clone(),
        validator: default_fallback_dashboard_reservation_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
