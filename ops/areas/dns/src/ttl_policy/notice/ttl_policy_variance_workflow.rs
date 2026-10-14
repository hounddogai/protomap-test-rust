use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves ttl policy variances through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct TtlPolicyVarianceCoordinator {
    pub store: Rc<dyn TtlPolicyVarianceStore>,
    pub validator: TtlPolicyVarianceValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: TtlPolicyVarianceEngine,
}

impl TtlPolicyVarianceCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("ttl policy variance {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(TtlPolicyVarianceStatus::Scheduled)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is scheduled", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("ttl policy variance {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(TtlPolicyVarianceStatus::Cancelled)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is cancelled: {}", code, reason));
        Ok(())
    }

    pub fn estimate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("ttl policy variance {} not found", code));
        };
        let result = self.engine.run(&item);
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

pub fn new_ttl_policy_variance_coordinator(
    store: Rc<dyn TtlPolicyVarianceStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> TtlPolicyVarianceCoordinator {
    TtlPolicyVarianceCoordinator {
        store: store.clone(),
        validator: default_ttl_policy_variance_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_ttl_policy_variance_engine(flags),
    }
}
