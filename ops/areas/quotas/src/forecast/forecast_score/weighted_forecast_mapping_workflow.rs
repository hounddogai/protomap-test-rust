use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves weighted forecast mappings through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct WeightedForecastMappingDesk {
    pub store: Rc<dyn WeightedForecastMappingStore>,
    pub validator: WeightedForecastMappingValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: WeightedForecastMappingEngine,
}

impl WeightedForecastMappingDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("weighted forecast mapping {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(WeightedForecastMappingStatus::Approved)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is approved", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("weighted forecast mapping {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(WeightedForecastMappingStatus::Withdrawn)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is withdrawn: {}", code, reason));
        Ok(())
    }

    pub fn evaluate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("weighted forecast mapping {} not found", code));
        };
        let result = self.engine.price(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
    }

    pub fn sweep(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.starts_at > cutoff {
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

pub fn new_weighted_forecast_mapping_desk(
    store: Rc<dyn WeightedForecastMappingStore>,
    clock: Rc<dyn clock::Clock>,
) -> WeightedForecastMappingDesk {
    WeightedForecastMappingDesk {
        store: store.clone(),
        validator: default_weighted_forecast_mapping_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_weighted_forecast_mapping_engine(),
    }
}
