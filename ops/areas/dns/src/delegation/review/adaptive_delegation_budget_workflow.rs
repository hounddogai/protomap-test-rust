use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves adaptive delegation budgets through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct AdaptiveDelegationBudgetCoordinator {
    pub store: Rc<dyn AdaptiveDelegationBudgetStore>,
    pub validator: AdaptiveDelegationBudgetValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: AdaptiveDelegationBudgetEngine,
}

impl AdaptiveDelegationBudgetCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("adaptive delegation budget {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(AdaptiveDelegationBudgetStatus::InReview)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is in review", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("adaptive delegation budget {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(AdaptiveDelegationBudgetStatus::Archived)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is archived: {}", code, reason));
        Ok(())
    }

    pub fn quote(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("adaptive delegation budget {} not found", code));
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

pub fn new_adaptive_delegation_budget_coordinator(
    store: Rc<dyn AdaptiveDelegationBudgetStore>,
    clock: Rc<dyn clock::Clock>,
) -> AdaptiveDelegationBudgetCoordinator {
    AdaptiveDelegationBudgetCoordinator {
        store: store.clone(),
        validator: default_adaptive_delegation_budget_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_adaptive_delegation_budget_engine(),
    }
}
