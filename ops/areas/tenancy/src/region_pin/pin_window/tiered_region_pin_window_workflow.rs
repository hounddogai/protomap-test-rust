use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves tiered region pin windows through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct TieredRegionPinWindowService {
    pub store: Rc<dyn TieredRegionPinWindowStore>,
    pub validator: TieredRegionPinWindowValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl TieredRegionPinWindowService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("tiered region pin window {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(TieredRegionPinWindowStatus::Pending)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is pending", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("tiered region pin window {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(TieredRegionPinWindowStatus::Closed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is closed: {}", code, reason));
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

pub fn new_tiered_region_pin_window_service(
    store: Rc<dyn TieredRegionPinWindowStore>,
    clock: Rc<dyn clock::Clock>,
) -> TieredRegionPinWindowService {
    TieredRegionPinWindowService {
        store: store.clone(),
        validator: default_tiered_region_pin_window_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
