use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves release estimates through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct ReleaseEstimateManager {
    pub store: Rc<dyn ReleaseEstimateStore>,
    pub validator: ReleaseEstimateValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl ReleaseEstimateManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("release estimate {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(ReleaseEstimateStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("release estimate {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(ReleaseEstimateStatus::Retired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is retired: {}", code, reason));
        Ok(())
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

pub fn new_release_estimate_manager(
    store: Rc<dyn ReleaseEstimateStore>,
    clock: Rc<dyn clock::Clock>,
) -> ReleaseEstimateManager {
    ReleaseEstimateManager {
        store: store.clone(),
        validator: default_release_estimate_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
