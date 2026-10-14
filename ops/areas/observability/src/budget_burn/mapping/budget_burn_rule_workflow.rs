use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves budget burn rules through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct BudgetBurnRuleService {
    pub store: Rc<dyn BudgetBurnRuleStore>,
    pub validator: BudgetBurnRuleValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl BudgetBurnRuleService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("budget burn rule {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(BudgetBurnRuleStatus::Scheduled)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is scheduled", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("budget burn rule {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(BudgetBurnRuleStatus::Cancelled)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is cancelled: {}", code, reason));
        Ok(())
    }

    pub fn scan(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.starts_at > cutoff {
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

pub fn new_budget_burn_rule_service(
    store: Rc<dyn BudgetBurnRuleStore>,
    clock: Rc<dyn clock::Clock>,
) -> BudgetBurnRuleService {
    BudgetBurnRuleService {
        store: store.clone(),
        validator: default_budget_burn_rule_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
