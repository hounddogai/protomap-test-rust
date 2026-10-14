use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a usage summary must pass. Check returns its problems, or none.
pub trait UsageSummaryRule: std::fmt::Debug {
    fn check(&self, item: &UsageSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageSummaryRequiredNameRule {
    pub message: String,
}

impl UsageSummaryRule for UsageSummaryRequiredNameRule {
    fn check(&self, item: &UsageSummary) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageSummaryRequiredCodeRule {
    pub message: String,
}

impl UsageSummaryRule for UsageSummaryRequiredCodeRule {
    fn check(&self, item: &UsageSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageSummaryCodeLengthRule {
    pub max_length: i64,
}

impl UsageSummaryRule for UsageSummaryCodeLengthRule {
    fn check(&self, item: &UsageSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageSummaryCodePrefixRule {
    pub prefix: String,
}

impl UsageSummaryRule for UsageSummaryCodePrefixRule {
    fn check(&self, item: &UsageSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageSummaryCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl UsageSummaryRule for UsageSummaryCeilingRangeRule {
    fn check(&self, item: &UsageSummary) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageSummaryLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsageSummaryRule for UsageSummaryLimitRangeRule {
    fn check(&self, item: &UsageSummary) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a usage summary.
#[derive(Debug, Clone)]
pub struct UsageSummaryValidator {
    pub rules: Vec<Rc<dyn UsageSummaryRule>>,
}

impl UsageSummaryValidator {
    pub fn validate(&self, item: &UsageSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &UsageSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_summary_validator() -> UsageSummaryValidator {
    UsageSummaryValidator {
        rules: vec![
            Rc::new(UsageSummaryRequiredNameRule {
                message: "usage summary needs this field".to_string(),
            }) as Rc<dyn UsageSummaryRule>,
            Rc::new(UsageSummaryRequiredCodeRule {
                message: "usage summary needs a value".to_string(),
            }),
            Rc::new(UsageSummaryCodeLengthRule { max_length: 120 }),
            Rc::new(UsageSummaryCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(UsageSummaryCeilingRangeRule { min: 9.4, max: 4998.5 }),
            Rc::new(UsageSummaryLimitRangeRule { min: 3600, max: 3600 }),
        ],
    }
}
