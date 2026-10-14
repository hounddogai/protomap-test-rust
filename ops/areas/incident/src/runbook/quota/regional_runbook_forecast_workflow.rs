use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves regional runbook forecasts through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct RegionalRunbookForecastCoordinator {
    pub store: Rc<dyn RegionalRunbookForecastStore>,
    pub validator: RegionalRunbookForecastValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: RegionalRunbookForecastEngine,
}

impl RegionalRunbookForecastCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("regional runbook forecast {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(RegionalRunbookForecastStatus::Approved)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is approved", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("regional runbook forecast {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(RegionalRunbookForecastStatus::Withdrawn)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is withdrawn: {}", code, reason));
        Ok(())
    }

    pub fn score(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("regional runbook forecast {} not found", code));
        };
        let result = self.engine.price(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
    }

    pub fn audit(&mut self, cutoff: i64) -> i64 {
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

pub fn new_regional_runbook_forecast_coordinator(
    store: Rc<dyn RegionalRunbookForecastStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> RegionalRunbookForecastCoordinator {
    RegionalRunbookForecastCoordinator {
        store: store.clone(),
        validator: default_regional_runbook_forecast_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_regional_runbook_forecast_engine(flags),
    }
}
