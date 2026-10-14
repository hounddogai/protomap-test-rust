use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves credential thresholds through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct CredentialThresholdManager {
    pub store: Rc<dyn CredentialThresholdStore>,
    pub validator: CredentialThresholdValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl CredentialThresholdManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("credential threshold {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(CredentialThresholdStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("credential threshold {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(CredentialThresholdStatus::Retired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is retired: {}", code, reason));
        Ok(())
    }

    pub fn review_stale(&mut self, cutoff: i64) -> i64 {
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

pub fn new_credential_threshold_manager(
    store: Rc<dyn CredentialThresholdStore>,
    clock: Rc<dyn clock::Clock>,
) -> CredentialThresholdManager {
    CredentialThresholdManager {
        store: store.clone(),
        validator: default_credential_threshold_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
