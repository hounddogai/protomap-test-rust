use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves default silence overrides through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct DefaultSilenceOverrideService {
    pub store: Rc<dyn DefaultSilenceOverrideStore>,
    pub validator: DefaultSilenceOverrideValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl DefaultSilenceOverrideService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("default silence override {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(DefaultSilenceOverrideStatus::Approved)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is approved", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("default silence override {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(DefaultSilenceOverrideStatus::Withdrawn)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is withdrawn: {}", code, reason));
        Ok(())
    }

    pub fn audit(&mut self, cutoff: i64) -> i64 {
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

pub fn new_default_silence_override_service(
    store: Rc<dyn DefaultSilenceOverrideStore>,
    clock: Rc<dyn clock::Clock>,
) -> DefaultSilenceOverrideService {
    DefaultSilenceOverrideService {
        store: store.clone(),
        validator: default_default_silence_override_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
