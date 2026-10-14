use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves default trust domain budgets through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct DefaultTrustDomainBudgetDesk {
    pub store: Rc<dyn DefaultTrustDomainBudgetStore>,
    pub validator: DefaultTrustDomainBudgetValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: DefaultTrustDomainBudgetEngine,
}

impl DefaultTrustDomainBudgetDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("default trust domain budget {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(DefaultTrustDomainBudgetStatus::Granted)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is granted", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("default trust domain budget {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(DefaultTrustDomainBudgetStatus::Expired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is expired: {}", code, reason));
        Ok(())
    }

    pub fn estimate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("default trust domain budget {} not found", code));
        };
        let result = self.engine.compute(&item);
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

pub fn new_default_trust_domain_budget_desk(
    store: Rc<dyn DefaultTrustDomainBudgetStore>,
    clock: Rc<dyn clock::Clock>,
) -> DefaultTrustDomainBudgetDesk {
    DefaultTrustDomainBudgetDesk {
        store: store.clone(),
        validator: default_default_trust_domain_budget_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_default_trust_domain_budget_engine(),
    }
}
