use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves fallback trust domain estimates through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct FallbackTrustDomainEstimateCoordinator {
    pub store: Rc<dyn FallbackTrustDomainEstimateStore>,
    pub validator: FallbackTrustDomainEstimateValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: FallbackTrustDomainEstimateEngine,
}

impl FallbackTrustDomainEstimateCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback trust domain estimate {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(FallbackTrustDomainEstimateStatus::Pending)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is pending", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback trust domain estimate {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(FallbackTrustDomainEstimateStatus::Closed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is closed: {}", code, reason));
        Ok(())
    }

    pub fn estimate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("fallback trust domain estimate {} not found", code));
        };
        let result = self.engine.price(&item);
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

pub fn new_fallback_trust_domain_estimate_coordinator(
    store: Rc<dyn FallbackTrustDomainEstimateStore>,
    clock: Rc<dyn clock::Clock>,
) -> FallbackTrustDomainEstimateCoordinator {
    FallbackTrustDomainEstimateCoordinator {
        store: store.clone(),
        validator: default_fallback_trust_domain_estimate_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_fallback_trust_domain_estimate_engine(),
    }
}
