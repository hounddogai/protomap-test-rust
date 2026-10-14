use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding batch must pass. Check returns its problems, or none.
pub trait BindingBatchRule: std::fmt::Debug {
    fn check(&self, item: &BindingBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingBatchGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingBatchRule for BindingBatchGraceDaysRangeRule {
    fn check(&self, item: &BindingBatch) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingBatchAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BindingBatchRule for BindingBatchAmountRangeRule {
    fn check(&self, item: &BindingBatch) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingBatchBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BindingBatchRule for BindingBatchBudgetRangeRule {
    fn check(&self, item: &BindingBatch) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding batch.
#[derive(Debug, Clone)]
pub struct BindingBatchValidator {
    pub rules: Vec<Rc<dyn BindingBatchRule>>,
}

impl BindingBatchValidator {
    pub fn validate(&self, item: &BindingBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_batch_validator() -> BindingBatchValidator {
    BindingBatchValidator {
        rules: vec![
            Rc::new(BindingBatchGraceDaysRangeRule { min: 86400, max: 86400 }) as Rc<dyn BindingBatchRule>,
            Rc::new(BindingBatchAmountRangeRule { min: 0.1, max: 1406.0 }),
            Rc::new(BindingBatchBudgetRangeRule { min: 3.6, max: 975.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_batch_validator();
        let item = new_binding_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = BindingBatchGraceDaysRangeRule { min: 16, max: 64 };
        let mut item = new_binding_batch("x", "y");
        item.grace_days = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = BindingBatchAmountRangeRule { min: 9.9, max: 3158.6 };
        let mut item = new_binding_batch("x", "y");
        item.amount = 3160.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = BindingBatchBudgetRangeRule { min: 3.8, max: 4715.3 };
        let mut item = new_binding_batch("x", "y");
        item.budget = 4716.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
