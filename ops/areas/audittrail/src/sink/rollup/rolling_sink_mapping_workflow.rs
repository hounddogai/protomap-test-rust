use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves rolling sink mappings through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct RollingSinkMappingDesk {
    pub store: Rc<dyn RollingSinkMappingStore>,
    pub validator: RollingSinkMappingValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl RollingSinkMappingDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("rolling sink mapping {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(RollingSinkMappingStatus::Granted)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is granted", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("rolling sink mapping {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(RollingSinkMappingStatus::Expired)
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

pub fn new_rolling_sink_mapping_desk(
    store: Rc<dyn RollingSinkMappingStore>,
    clock: Rc<dyn clock::Clock>,
) -> RollingSinkMappingDesk {
    RollingSinkMappingDesk {
        store: store.clone(),
        validator: default_rolling_sink_mapping_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
