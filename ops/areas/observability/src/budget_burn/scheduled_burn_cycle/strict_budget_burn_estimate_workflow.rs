use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves strict budget burn estimates through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct StrictBudgetBurnEstimateDesk {
    pub store: Rc<dyn StrictBudgetBurnEstimateStore>,
    pub validator: StrictBudgetBurnEstimateValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl StrictBudgetBurnEstimateDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("strict budget burn estimate {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(StrictBudgetBurnEstimateStatus::InReview)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is in review", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("strict budget burn estimate {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(StrictBudgetBurnEstimateStatus::Archived)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is archived: {}", code, reason));
        Ok(())
    }

    pub fn sweep(&mut self, cutoff: i64) -> i64 {
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

pub fn new_strict_budget_burn_estimate_desk(
    store: Rc<dyn StrictBudgetBurnEstimateStore>,
    clock: Rc<dyn clock::Clock>,
) -> StrictBudgetBurnEstimateDesk {
    StrictBudgetBurnEstimateDesk {
        store: store.clone(),
        validator: default_strict_budget_burn_estimate_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
