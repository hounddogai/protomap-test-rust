use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast budget must pass. Check returns its problems, or none.
pub trait ForecastBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ForecastBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBudgetScopeLengthRule {
    pub max_length: i64,
}

impl ForecastBudgetRule for ForecastBudgetScopeLengthRule {
    fn check(&self, item: &ForecastBudget) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBudgetRequiredCodeRule {
    pub message: String,
}

impl ForecastBudgetRule for ForecastBudgetRequiredCodeRule {
    fn check(&self, item: &ForecastBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBudgetRequiredLabelRule {
    pub message: String,
}

impl ForecastBudgetRule for ForecastBudgetRequiredLabelRule {
    fn check(&self, item: &ForecastBudget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBudgetLabelLengthRule {
    pub max_length: i64,
}

impl ForecastBudgetRule for ForecastBudgetLabelLengthRule {
    fn check(&self, item: &ForecastBudget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBudgetCodeLengthRule {
    pub max_length: i64,
}

impl ForecastBudgetRule for ForecastBudgetCodeLengthRule {
    fn check(&self, item: &ForecastBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBudgetCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastBudgetRule for ForecastBudgetCostRangeRule {
    fn check(&self, item: &ForecastBudget) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast budget.
#[derive(Debug, Clone)]
pub struct ForecastBudgetValidator {
    pub rules: Vec<Rc<dyn ForecastBudgetRule>>,
}

impl ForecastBudgetValidator {
    pub fn validate(&self, item: &ForecastBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_budget_validator() -> ForecastBudgetValidator {
    ForecastBudgetValidator {
        rules: vec![
            Rc::new(ForecastBudgetScopeLengthRule { max_length: 16 }) as Rc<dyn ForecastBudgetRule>,
            Rc::new(ForecastBudgetRequiredCodeRule {
                message: "forecast budget needs a value".to_string(),
            }),
            Rc::new(ForecastBudgetRequiredLabelRule {
                message: "forecast budget needs an entry".to_string(),
            }),
            Rc::new(ForecastBudgetLabelLengthRule { max_length: 32 }),
            Rc::new(ForecastBudgetCodeLengthRule { max_length: 3600 }),
            Rc::new(ForecastBudgetCostRangeRule { min: 6.2, max: 4017.2 }),
        ],
    }
}
