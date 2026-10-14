use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves pending overage bundles through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct PendingOverageBundleDesk {
    pub store: Rc<dyn PendingOverageBundleStore>,
    pub validator: PendingOverageBundleValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: PendingOverageBundleEngine,
}

impl PendingOverageBundleDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("pending overage bundle {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(PendingOverageBundleStatus::Pending)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is pending", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("pending overage bundle {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(PendingOverageBundleStatus::Closed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is closed: {}", code, reason));
        Ok(())
    }

    pub fn score(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("pending overage bundle {} not found", code));
        };
        let result = self.engine.run(&item);
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

pub fn new_pending_overage_bundle_desk(
    store: Rc<dyn PendingOverageBundleStore>,
    clock: Rc<dyn clock::Clock>,
) -> PendingOverageBundleDesk {
    PendingOverageBundleDesk {
        store: store.clone(),
        validator: default_pending_overage_bundle_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_pending_overage_bundle_engine(),
    }
}
