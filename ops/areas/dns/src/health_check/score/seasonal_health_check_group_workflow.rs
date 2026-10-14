use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves seasonal health check groups through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct SeasonalHealthCheckGroupDesk {
    pub store: Rc<dyn SeasonalHealthCheckGroupStore>,
    pub validator: SeasonalHealthCheckGroupValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: SeasonalHealthCheckGroupEngine,
}

impl SeasonalHealthCheckGroupDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("seasonal health check group {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(SeasonalHealthCheckGroupStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("seasonal health check group {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(SeasonalHealthCheckGroupStatus::Retired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is retired: {}", code, reason));
        Ok(())
    }

    pub fn quote(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("seasonal health check group {} not found", code));
        };
        let result = self.engine.evaluate(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
    }

    pub fn scan(&mut self, cutoff: i64) -> i64 {
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

pub fn new_seasonal_health_check_group_desk(
    store: Rc<dyn SeasonalHealthCheckGroupStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> SeasonalHealthCheckGroupDesk {
    SeasonalHealthCheckGroupDesk {
        store: store.clone(),
        validator: default_seasonal_health_check_group_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_seasonal_health_check_group_engine(flags),
    }
}
