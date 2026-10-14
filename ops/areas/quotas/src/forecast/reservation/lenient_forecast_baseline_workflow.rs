use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves lenient forecast baselines through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct LenientForecastBaselineDesk {
    pub store: Rc<dyn LenientForecastBaselineStore>,
    pub validator: LenientForecastBaselineValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl LenientForecastBaselineDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("lenient forecast baseline {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(LenientForecastBaselineStatus::Scheduled)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is scheduled", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("lenient forecast baseline {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(LenientForecastBaselineStatus::Cancelled)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is cancelled: {}", code, reason));
        Ok(())
    }

    pub fn sweep(&mut self, cutoff: i64) -> i64 {
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

pub fn new_lenient_forecast_baseline_desk(
    store: Rc<dyn LenientForecastBaselineStore>,
    clock: Rc<dyn clock::Clock>,
) -> LenientForecastBaselineDesk {
    LenientForecastBaselineDesk {
        store: store.clone(),
        validator: default_lenient_forecast_baseline_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
