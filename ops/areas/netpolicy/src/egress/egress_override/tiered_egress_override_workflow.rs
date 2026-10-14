use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves tiered egress overrides through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct TieredEgressOverrideService {
    pub store: Rc<dyn TieredEgressOverrideStore>,
    pub validator: TieredEgressOverrideValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl TieredEgressOverrideService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("tiered egress override {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(TieredEgressOverrideStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("tiered egress override {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(TieredEgressOverrideStatus::Retired)
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
            if item.expires_at > cutoff {
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

pub fn new_tiered_egress_override_service(
    store: Rc<dyn TieredEgressOverrideStore>,
    clock: Rc<dyn clock::Clock>,
) -> TieredEgressOverrideService {
    TieredEgressOverrideService {
        store: store.clone(),
        validator: default_tiered_egress_override_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
