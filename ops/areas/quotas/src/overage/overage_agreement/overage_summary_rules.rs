use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage summary must pass. Check returns its problems, or none.
pub trait OverageSummaryRule: std::fmt::Debug {
    fn check(&self, item: &OverageSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageSummaryMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl OverageSummaryRule for OverageSummaryMarginRangeRule {
    fn check(&self, item: &OverageSummary) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageSummaryCodePrefixRule {
    pub prefix: String,
}

impl OverageSummaryRule for OverageSummaryCodePrefixRule {
    fn check(&self, item: &OverageSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageSummaryRequiredTitleRule {
    pub message: String,
}

impl OverageSummaryRule for OverageSummaryRequiredTitleRule {
    fn check(&self, item: &OverageSummary) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageSummaryCodeLengthRule {
    pub max_length: i64,
}

impl OverageSummaryRule for OverageSummaryCodeLengthRule {
    fn check(&self, item: &OverageSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageSummaryTitleLengthRule {
    pub max_length: i64,
}

impl OverageSummaryRule for OverageSummaryTitleLengthRule {
    fn check(&self, item: &OverageSummary) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage summary.
#[derive(Debug, Clone)]
pub struct OverageSummaryValidator {
    pub rules: Vec<Rc<dyn OverageSummaryRule>>,
}

impl OverageSummaryValidator {
    pub fn validate(&self, item: &OverageSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &OverageSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_summary_validator() -> OverageSummaryValidator {
    OverageSummaryValidator {
        rules: vec![
            Rc::new(OverageSummaryMarginRangeRule { min: 5.0, max: 3648.8 }) as Rc<dyn OverageSummaryRule>,
            Rc::new(OverageSummaryCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(OverageSummaryRequiredTitleRule {
                message: "overage summary needs an entry".to_string(),
            }),
            Rc::new(OverageSummaryCodeLengthRule { max_length: 64 }),
            Rc::new(OverageSummaryTitleLengthRule { max_length: 64 }),
        ],
    }
}
