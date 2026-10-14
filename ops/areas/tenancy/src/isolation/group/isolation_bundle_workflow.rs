use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves isolation bundles through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct IsolationBundleCoordinator {
    pub store: Rc<dyn IsolationBundleStore>,
    pub validator: IsolationBundleValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: IsolationBundleEngine,
}

impl IsolationBundleCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("isolation bundle {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(IsolationBundleStatus::Scheduled)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is scheduled", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("isolation bundle {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(IsolationBundleStatus::Cancelled)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is cancelled: {}", code, reason));
        Ok(())
    }

    pub fn evaluate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("isolation bundle {} not found", code));
        };
        let result = self.engine.compute(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
    }

    pub fn sweep(&mut self, cutoff: i64) -> i64 {
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

pub fn new_isolation_bundle_coordinator(
    store: Rc<dyn IsolationBundleStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> IsolationBundleCoordinator {
    IsolationBundleCoordinator {
        store: store.clone(),
        validator: default_isolation_bundle_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_isolation_bundle_engine(flags),
    }
}
