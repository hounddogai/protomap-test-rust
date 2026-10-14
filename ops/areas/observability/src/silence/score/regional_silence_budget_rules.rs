use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional silence budget must pass. Check returns its problems, or none.
pub trait RegionalSilenceBudgetRule: std::fmt::Debug {
    fn check(&self, item: &RegionalSilenceBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceBudgetRequiredRegionRule {
    pub message: String,
}

impl RegionalSilenceBudgetRule for RegionalSilenceBudgetRequiredRegionRule {
    fn check(&self, item: &RegionalSilenceBudget) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceBudgetRegionLengthRule {
    pub max_length: i64,
}

impl RegionalSilenceBudgetRule for RegionalSilenceBudgetRegionLengthRule {
    fn check(&self, item: &RegionalSilenceBudget) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceBudgetBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalSilenceBudgetRule for RegionalSilenceBudgetBudgetRangeRule {
    fn check(&self, item: &RegionalSilenceBudget) -> Vec<String> {
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
pub struct RegionalSilenceBudgetRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalSilenceBudgetRule for RegionalSilenceBudgetRevisionRangeRule {
    fn check(&self, item: &RegionalSilenceBudget) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceBudgetRequiredLabelRule {
    pub message: String,
}

impl RegionalSilenceBudgetRule for RegionalSilenceBudgetRequiredLabelRule {
    fn check(&self, item: &RegionalSilenceBudget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceBudgetCodePrefixRule {
    pub prefix: String,
}

impl RegionalSilenceBudgetRule for RegionalSilenceBudgetCodePrefixRule {
    fn check(&self, item: &RegionalSilenceBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional silence budget.
#[derive(Debug, Clone)]
pub struct RegionalSilenceBudgetValidator {
    pub rules: Vec<Rc<dyn RegionalSilenceBudgetRule>>,
}

impl RegionalSilenceBudgetValidator {
    pub fn validate(&self, item: &RegionalSilenceBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalSilenceBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalSilenceBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_silence_budget_validator() -> RegionalSilenceBudgetValidator {
    RegionalSilenceBudgetValidator {
        rules: vec![
            Rc::new(RegionalSilenceBudgetRequiredRegionRule {
                message: "regional silence budget needs a value".to_string(),
            }) as Rc<dyn RegionalSilenceBudgetRule>,
            Rc::new(RegionalSilenceBudgetRegionLengthRule { max_length: 1000 }),
            Rc::new(RegionalSilenceBudgetBudgetRangeRule { min: 5.4, max: 1286.6 }),
            Rc::new(RegionalSilenceBudgetRevisionRangeRule { min: 32, max: 255 }),
            Rc::new(RegionalSilenceBudgetRequiredLabelRule {
                message: "regional silence budget needs an entry".to_string(),
            }),
            Rc::new(RegionalSilenceBudgetCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
