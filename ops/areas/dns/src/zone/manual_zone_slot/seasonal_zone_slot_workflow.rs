use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves seasonal zone slots through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct SeasonalZoneSlotService {
    pub store: Rc<dyn SeasonalZoneSlotStore>,
    pub validator: SeasonalZoneSlotValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl SeasonalZoneSlotService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("seasonal zone slot {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(SeasonalZoneSlotStatus::Granted)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is granted", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("seasonal zone slot {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(SeasonalZoneSlotStatus::Expired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is expired: {}", code, reason));
        Ok(())
    }

    pub fn review_stale(&mut self, cutoff: i64) -> i64 {
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

pub fn new_seasonal_zone_slot_service(
    store: Rc<dyn SeasonalZoneSlotStore>,
    clock: Rc<dyn clock::Clock>,
) -> SeasonalZoneSlotService {
    SeasonalZoneSlotService {
        store: store.clone(),
        validator: default_seasonal_zone_slot_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
