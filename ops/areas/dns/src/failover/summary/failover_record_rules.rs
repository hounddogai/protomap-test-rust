use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover record must pass. Check returns its problems, or none.
pub trait FailoverRecordRule: std::fmt::Debug {
    fn check(&self, item: &FailoverRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRecordRequiredLabelRule {
    pub message: String,
}

impl FailoverRecordRule for FailoverRecordRequiredLabelRule {
    fn check(&self, item: &FailoverRecord) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRecordRequiredCodeRule {
    pub message: String,
}

impl FailoverRecordRule for FailoverRecordRequiredCodeRule {
    fn check(&self, item: &FailoverRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRecordLabelLengthRule {
    pub max_length: i64,
}

impl FailoverRecordRule for FailoverRecordLabelLengthRule {
    fn check(&self, item: &FailoverRecord) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRecordCodePrefixRule {
    pub prefix: String,
}

impl FailoverRecordRule for FailoverRecordCodePrefixRule {
    fn check(&self, item: &FailoverRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRecordCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FailoverRecordRule for FailoverRecordCostRangeRule {
    fn check(&self, item: &FailoverRecord) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRecordDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FailoverRecordRule for FailoverRecordDiscountRangeRule {
    fn check(&self, item: &FailoverRecord) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover record.
#[derive(Debug, Clone)]
pub struct FailoverRecordValidator {
    pub rules: Vec<Rc<dyn FailoverRecordRule>>,
}

impl FailoverRecordValidator {
    pub fn validate(&self, item: &FailoverRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FailoverRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_record_validator() -> FailoverRecordValidator {
    FailoverRecordValidator {
        rules: vec![
            Rc::new(FailoverRecordRequiredLabelRule {
                message: "failover record needs a value".to_string(),
            }) as Rc<dyn FailoverRecordRule>,
            Rc::new(FailoverRecordRequiredCodeRule {
                message: "failover record needs an entry".to_string(),
            }),
            Rc::new(FailoverRecordLabelLengthRule { max_length: 80 }),
            Rc::new(FailoverRecordCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(FailoverRecordCostRangeRule { min: 4.4, max: 2704.2 }),
            Rc::new(FailoverRecordDiscountRangeRule { min: 2.7, max: 4187.5 }),
        ],
    }
}
