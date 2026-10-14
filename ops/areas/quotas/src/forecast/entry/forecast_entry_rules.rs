use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast entry must pass. Check returns its problems, or none.
pub trait ForecastEntryRule: std::fmt::Debug {
    fn check(&self, item: &ForecastEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastEntryRequiredUnitRule {
    pub message: String,
}

impl ForecastEntryRule for ForecastEntryRequiredUnitRule {
    fn check(&self, item: &ForecastEntry) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastEntryPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastEntryRule for ForecastEntryPriorityRangeRule {
    fn check(&self, item: &ForecastEntry) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastEntryRequiredLabelRule {
    pub message: String,
}

impl ForecastEntryRule for ForecastEntryRequiredLabelRule {
    fn check(&self, item: &ForecastEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastEntryCodePrefixRule {
    pub prefix: String,
}

impl ForecastEntryRule for ForecastEntryCodePrefixRule {
    fn check(&self, item: &ForecastEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast entry.
#[derive(Debug, Clone)]
pub struct ForecastEntryValidator {
    pub rules: Vec<Rc<dyn ForecastEntryRule>>,
}

impl ForecastEntryValidator {
    pub fn validate(&self, item: &ForecastEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_entry_validator() -> ForecastEntryValidator {
    ForecastEntryValidator {
        rules: vec![
            Rc::new(ForecastEntryRequiredUnitRule {
                message: "forecast entry needs an entry".to_string(),
            }) as Rc<dyn ForecastEntryRule>,
            Rc::new(ForecastEntryPriorityRangeRule { min: 16, max: 64 }),
            Rc::new(ForecastEntryRequiredLabelRule {
                message: "forecast entry needs this field".to_string(),
            }),
            Rc::new(ForecastEntryCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
