use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves adaptive approval agreements through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct AdaptiveApprovalAgreementService {
    pub store: Rc<dyn AdaptiveApprovalAgreementStore>,
    pub validator: AdaptiveApprovalAgreementValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: AdaptiveApprovalAgreementEngine,
}

impl AdaptiveApprovalAgreementService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("adaptive approval agreement {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(AdaptiveApprovalAgreementStatus::Granted)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is granted", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("adaptive approval agreement {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(AdaptiveApprovalAgreementStatus::Expired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is expired: {}", code, reason));
        Ok(())
    }

    pub fn quote(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("adaptive approval agreement {} not found", code));
        };
        let result = self.engine.price(&item);
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

pub fn new_adaptive_approval_agreement_service(
    store: Rc<dyn AdaptiveApprovalAgreementStore>,
    clock: Rc<dyn clock::Clock>,
) -> AdaptiveApprovalAgreementService {
    AdaptiveApprovalAgreementService {
        store: store.clone(),
        validator: default_adaptive_approval_agreement_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_adaptive_approval_agreement_engine(),
    }
}
