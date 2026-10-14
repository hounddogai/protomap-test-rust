use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves trust domain agreements through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct TrustDomainAgreementCoordinator {
    pub store: Rc<dyn TrustDomainAgreementStore>,
    pub validator: TrustDomainAgreementValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl TrustDomainAgreementCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("trust domain agreement {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(TrustDomainAgreementStatus::Scheduled)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is scheduled", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("trust domain agreement {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(TrustDomainAgreementStatus::Cancelled)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is cancelled: {}", code, reason));
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

pub fn new_trust_domain_agreement_coordinator(
    store: Rc<dyn TrustDomainAgreementStore>,
    clock: Rc<dyn clock::Clock>,
) -> TrustDomainAgreementCoordinator {
    TrustDomainAgreementCoordinator {
        store: store.clone(),
        validator: default_trust_domain_agreement_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
