use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves fallback region pin mappings through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct FallbackRegionPinMappingService {
    pub store: Rc<dyn FallbackRegionPinMappingStore>,
    pub validator: FallbackRegionPinMappingValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: FallbackRegionPinMappingEngine,
}

impl FallbackRegionPinMappingService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback region pin mapping {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(FallbackRegionPinMappingStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback region pin mapping {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(FallbackRegionPinMappingStatus::Retired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is retired: {}", code, reason));
        Ok(())
    }

    pub fn quote(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("fallback region pin mapping {} not found", code));
        };
        let result = self.engine.compute(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
    }

    pub fn audit(&mut self, cutoff: i64) -> i64 {
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

pub fn new_fallback_region_pin_mapping_service(
    store: Rc<dyn FallbackRegionPinMappingStore>,
    clock: Rc<dyn clock::Clock>,
) -> FallbackRegionPinMappingService {
    FallbackRegionPinMappingService {
        store: store.clone(),
        validator: default_fallback_region_pin_mapping_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_fallback_region_pin_mapping_engine(),
    }
}
