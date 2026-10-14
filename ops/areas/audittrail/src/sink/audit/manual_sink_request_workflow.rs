use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves manual sink requests through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct ManualSinkRequestCoordinator {
    pub store: Rc<dyn ManualSinkRequestStore>,
    pub validator: ManualSinkRequestValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: ManualSinkRequestEngine,
}

impl ManualSinkRequestCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("manual sink request {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(ManualSinkRequestStatus::Granted)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is granted", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("manual sink request {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(ManualSinkRequestStatus::Expired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is expired: {}", code, reason));
        Ok(())
    }

    pub fn estimate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("manual sink request {} not found", code));
        };
        let result = self.engine.evaluate(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
    }

    pub fn audit(&mut self, cutoff: i64) -> i64 {
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

pub fn new_manual_sink_request_coordinator(
    store: Rc<dyn ManualSinkRequestStore>,
    clock: Rc<dyn clock::Clock>,
) -> ManualSinkRequestCoordinator {
    ManualSinkRequestCoordinator {
        store: store.clone(),
        validator: default_manual_sink_request_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_manual_sink_request_engine(),
    }
}
