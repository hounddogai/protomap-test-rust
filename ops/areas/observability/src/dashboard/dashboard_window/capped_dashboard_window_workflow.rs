use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves capped dashboard windows through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct CappedDashboardWindowManager {
    pub store: Rc<dyn CappedDashboardWindowStore>,
    pub validator: CappedDashboardWindowValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: CappedDashboardWindowEngine,
}

impl CappedDashboardWindowManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("capped dashboard window {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(CappedDashboardWindowStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("capped dashboard window {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(CappedDashboardWindowStatus::Retired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is retired: {}", code, reason));
        Ok(())
    }

    pub fn score(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("capped dashboard window {} not found", code));
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

pub fn new_capped_dashboard_window_manager(
    store: Rc<dyn CappedDashboardWindowStore>,
    clock: Rc<dyn clock::Clock>,
) -> CappedDashboardWindowManager {
    CappedDashboardWindowManager {
        store: store.clone(),
        validator: default_capped_dashboard_window_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_capped_dashboard_window_engine(),
    }
}
