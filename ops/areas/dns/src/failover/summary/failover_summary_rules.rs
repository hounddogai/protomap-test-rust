use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover summary must pass. Check returns its problems, or none.
pub trait FailoverSummaryRule: std::fmt::Debug {
    fn check(&self, item: &FailoverSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSummaryCodeLengthRule {
    pub max_length: i64,
}

impl FailoverSummaryRule for FailoverSummaryCodeLengthRule {
    fn check(&self, item: &FailoverSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSummaryWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverSummaryRule for FailoverSummaryWindowDaysRangeRule {
    fn check(&self, item: &FailoverSummary) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSummaryLabelLengthRule {
    pub max_length: i64,
}

impl FailoverSummaryRule for FailoverSummaryLabelLengthRule {
    fn check(&self, item: &FailoverSummary) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSummarySummaryLengthRule {
    pub max_length: i64,
}

impl FailoverSummaryRule for FailoverSummarySummaryLengthRule {
    fn check(&self, item: &FailoverSummary) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSummaryRequiredIssuerRule {
    pub message: String,
}

impl FailoverSummaryRule for FailoverSummaryRequiredIssuerRule {
    fn check(&self, item: &FailoverSummary) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover summary.
#[derive(Debug, Clone)]
pub struct FailoverSummaryValidator {
    pub rules: Vec<Rc<dyn FailoverSummaryRule>>,
}

impl FailoverSummaryValidator {
    pub fn validate(&self, item: &FailoverSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_summary_validator() -> FailoverSummaryValidator {
    FailoverSummaryValidator {
        rules: vec![
            Rc::new(FailoverSummaryCodeLengthRule { max_length: 16 }) as Rc<dyn FailoverSummaryRule>,
            Rc::new(FailoverSummaryWindowDaysRangeRule { min: 32, max: 3600 }),
            Rc::new(FailoverSummaryLabelLengthRule { max_length: 3600 }),
            Rc::new(FailoverSummarySummaryLengthRule { max_length: 16 }),
            Rc::new(FailoverSummaryRequiredIssuerRule {
                message: "failover summary needs this field".to_string(),
            }),
        ],
    }
}
