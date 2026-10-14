use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves budget burn rollups through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct BudgetBurnRollupManager {
    pub store: Rc<dyn BudgetBurnRollupStore>,
    pub validator: BudgetBurnRollupValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl BudgetBurnRollupManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("budget burn rollup {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(BudgetBurnRollupStatus::Approved)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is approved", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("budget burn rollup {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(BudgetBurnRollupStatus::Withdrawn)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is withdrawn: {}", code, reason));
        Ok(())
    }

    pub fn scan(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.ends_at > cutoff {
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

pub fn new_budget_burn_rollup_manager(
    store: Rc<dyn BudgetBurnRollupStore>,
    clock: Rc<dyn clock::Clock>,
) -> BudgetBurnRollupManager {
    BudgetBurnRollupManager {
        store: store.clone(),
        validator: default_budget_burn_rollup_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
