use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves forecast preferences through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct ForecastPreferenceCoordinator {
    pub store: Rc<dyn ForecastPreferenceStore>,
    pub validator: ForecastPreferenceValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl ForecastPreferenceCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("forecast preference {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(ForecastPreferenceStatus::Pending)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is pending", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("forecast preference {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(ForecastPreferenceStatus::Closed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is closed: {}", code, reason));
        Ok(())
    }

    pub fn scan(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.updated_at > cutoff {
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

pub fn new_forecast_preference_coordinator(
    store: Rc<dyn ForecastPreferenceStore>,
    clock: Rc<dyn clock::Clock>,
) -> ForecastPreferenceCoordinator {
    ForecastPreferenceCoordinator {
        store: store.clone(),
        validator: default_forecast_preference_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
