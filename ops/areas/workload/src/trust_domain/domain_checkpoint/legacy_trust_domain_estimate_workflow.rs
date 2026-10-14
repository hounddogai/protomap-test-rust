use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves legacy trust domain estimates through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct LegacyTrustDomainEstimateService {
    pub store: Rc<dyn LegacyTrustDomainEstimateStore>,
    pub validator: LegacyTrustDomainEstimateValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: LegacyTrustDomainEstimateEngine,
}

impl LegacyTrustDomainEstimateService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("legacy trust domain estimate {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(LegacyTrustDomainEstimateStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("legacy trust domain estimate {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(LegacyTrustDomainEstimateStatus::Retired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is retired: {}", code, reason));
        Ok(())
    }

    pub fn evaluate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("legacy trust domain estimate {} not found", code));
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

pub fn new_legacy_trust_domain_estimate_service(
    store: Rc<dyn LegacyTrustDomainEstimateStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> LegacyTrustDomainEstimateService {
    LegacyTrustDomainEstimateService {
        store: store.clone(),
        validator: default_legacy_trust_domain_estimate_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_legacy_trust_domain_estimate_engine(flags),
    }
}
