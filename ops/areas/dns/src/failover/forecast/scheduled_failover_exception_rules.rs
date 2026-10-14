use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled failover exception must pass. Check returns its problems, or none.
pub trait ScheduledFailoverExceptionRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledFailoverException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverExceptionRequiredContactEmailRule {
    pub message: String,
}

impl ScheduledFailoverExceptionRule for ScheduledFailoverExceptionRequiredContactEmailRule {
    fn check(&self, item: &ScheduledFailoverException) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverExceptionRequiredLabelRule {
    pub message: String,
}

impl ScheduledFailoverExceptionRule for ScheduledFailoverExceptionRequiredLabelRule {
    fn check(&self, item: &ScheduledFailoverException) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverExceptionAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledFailoverExceptionRule for ScheduledFailoverExceptionAmountRangeRule {
    fn check(&self, item: &ScheduledFailoverException) -> Vec<String> {
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
pub struct ScheduledFailoverExceptionThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledFailoverExceptionRule for ScheduledFailoverExceptionThresholdRangeRule {
    fn check(&self, item: &ScheduledFailoverException) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverExceptionMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledFailoverExceptionRule for ScheduledFailoverExceptionMarginRangeRule {
    fn check(&self, item: &ScheduledFailoverException) -> Vec<String> {
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
pub struct ScheduledFailoverExceptionCodePrefixRule {
    pub prefix: String,
}

impl ScheduledFailoverExceptionRule for ScheduledFailoverExceptionCodePrefixRule {
    fn check(&self, item: &ScheduledFailoverException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled failover exception.
#[derive(Debug, Clone)]
pub struct ScheduledFailoverExceptionValidator {
    pub rules: Vec<Rc<dyn ScheduledFailoverExceptionRule>>,
}

impl ScheduledFailoverExceptionValidator {
    pub fn validate(&self, item: &ScheduledFailoverException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledFailoverException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledFailoverException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_failover_exception_validator() -> ScheduledFailoverExceptionValidator {
    ScheduledFailoverExceptionValidator {
        rules: vec![
            Rc::new(ScheduledFailoverExceptionRequiredContactEmailRule {
                message: "scheduled failover exception needs a value".to_string(),
            }) as Rc<dyn ScheduledFailoverExceptionRule>,
            Rc::new(ScheduledFailoverExceptionRequiredLabelRule {
                message: "scheduled failover exception needs a value".to_string(),
            }),
            Rc::new(ScheduledFailoverExceptionAmountRangeRule { min: 2.5, max: 2473.5 }),
            Rc::new(ScheduledFailoverExceptionThresholdRangeRule { min: 4.5, max: 3273.0 }),
            Rc::new(ScheduledFailoverExceptionMarginRangeRule { min: 7.2, max: 4147.9 }),
            Rc::new(ScheduledFailoverExceptionCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}
