use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement budget must pass. Check returns its problems, or none.
pub trait MeasurementBudgetRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBudgetCodeLengthRule {
    pub max_length: i64,
}

impl MeasurementBudgetRule for MeasurementBudgetCodeLengthRule {
    fn check(&self, item: &MeasurementBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBudgetBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementBudgetRule for MeasurementBudgetBudgetRangeRule {
    fn check(&self, item: &MeasurementBudget) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBudgetDescriptionLengthRule {
    pub max_length: i64,
}

impl MeasurementBudgetRule for MeasurementBudgetDescriptionLengthRule {
    fn check(&self, item: &MeasurementBudget) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBudgetGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementBudgetRule for MeasurementBudgetGraceDaysRangeRule {
    fn check(&self, item: &MeasurementBudget) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBudgetAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementBudgetRule for MeasurementBudgetAmountRangeRule {
    fn check(&self, item: &MeasurementBudget) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBudgetRequiredDescriptionRule {
    pub message: String,
}

impl MeasurementBudgetRule for MeasurementBudgetRequiredDescriptionRule {
    fn check(&self, item: &MeasurementBudget) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement budget.
#[derive(Debug, Clone)]
pub struct MeasurementBudgetValidator {
    pub rules: Vec<Rc<dyn MeasurementBudgetRule>>,
}

impl MeasurementBudgetValidator {
    pub fn validate(&self, item: &MeasurementBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MeasurementBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_budget_validator() -> MeasurementBudgetValidator {
    MeasurementBudgetValidator {
        rules: vec![
            Rc::new(MeasurementBudgetCodeLengthRule { max_length: 1000 }) as Rc<dyn MeasurementBudgetRule>,
            Rc::new(MeasurementBudgetBudgetRangeRule { min: 6.7, max: 3388.7 }),
            Rc::new(MeasurementBudgetDescriptionLengthRule { max_length: 64 }),
            Rc::new(MeasurementBudgetGraceDaysRangeRule { min: 64, max: 64 }),
            Rc::new(MeasurementBudgetAmountRangeRule { min: 7.1, max: 2597.0 }),
            Rc::new(MeasurementBudgetRequiredDescriptionRule {
                message: "measurement budget needs a value".to_string(),
            }),
        ],
    }
}
