use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn slot must pass. Check returns its problems, or none.
pub trait BudgetBurnSlotRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnSlotRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnSlotRule for BudgetBurnSlotRotationDaysRangeRule {
    fn check(&self, item: &BudgetBurnSlot) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnSlotCountryLengthRule {
    pub max_length: i64,
}

impl BudgetBurnSlotRule for BudgetBurnSlotCountryLengthRule {
    fn check(&self, item: &BudgetBurnSlot) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnSlotBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnSlotRule for BudgetBurnSlotBatchSizeRangeRule {
    fn check(&self, item: &BudgetBurnSlot) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a budget burn slot.
#[derive(Debug, Clone)]
pub struct BudgetBurnSlotValidator {
    pub rules: Vec<Rc<dyn BudgetBurnSlotRule>>,
}

impl BudgetBurnSlotValidator {
    pub fn validate(&self, item: &BudgetBurnSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BudgetBurnSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnSlot]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_budget_burn_slot_validator() -> BudgetBurnSlotValidator {
    BudgetBurnSlotValidator {
        rules: vec![
            Rc::new(BudgetBurnSlotRotationDaysRangeRule { min: 120, max: 3600 }) as Rc<dyn BudgetBurnSlotRule>,
            Rc::new(BudgetBurnSlotCountryLengthRule { max_length: 32 }),
            Rc::new(BudgetBurnSlotBatchSizeRangeRule { min: 80, max: 1000 }),
        ],
    }
}
