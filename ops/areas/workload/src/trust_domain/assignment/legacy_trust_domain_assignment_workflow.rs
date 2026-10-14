use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves legacy trust domain assignments through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct LegacyTrustDomainAssignmentService {
    pub store: Rc<dyn LegacyTrustDomainAssignmentStore>,
    pub validator: LegacyTrustDomainAssignmentValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: LegacyTrustDomainAssignmentEngine,
}

impl LegacyTrustDomainAssignmentService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("legacy trust domain assignment {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(LegacyTrustDomainAssignmentStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("legacy trust domain assignment {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(LegacyTrustDomainAssignmentStatus::Retired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is retired: {}", code, reason));
        Ok(())
    }

    pub fn estimate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("legacy trust domain assignment {} not found", code));
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

pub fn new_legacy_trust_domain_assignment_service(
    store: Rc<dyn LegacyTrustDomainAssignmentStore>,
    clock: Rc<dyn clock::Clock>,
) -> LegacyTrustDomainAssignmentService {
    LegacyTrustDomainAssignmentService {
        store: store.clone(),
        validator: default_legacy_trust_domain_assignment_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_legacy_trust_domain_assignment_engine(),
    }
}
