use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted identity record must pass. Check returns its problems, or none.
pub trait WeightedIdentityRecordRule: std::fmt::Debug {
    fn check(&self, item: &WeightedIdentityRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityRecordLabelLengthRule {
    pub max_length: i64,
}

impl WeightedIdentityRecordRule for WeightedIdentityRecordLabelLengthRule {
    fn check(&self, item: &WeightedIdentityRecord) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityRecordRequiredLabelRule {
    pub message: String,
}

impl WeightedIdentityRecordRule for WeightedIdentityRecordRequiredLabelRule {
    fn check(&self, item: &WeightedIdentityRecord) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityRecordCodePrefixRule {
    pub prefix: String,
}

impl WeightedIdentityRecordRule for WeightedIdentityRecordCodePrefixRule {
    fn check(&self, item: &WeightedIdentityRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityRecordAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl WeightedIdentityRecordRule for WeightedIdentityRecordAllowedCodeRule {
    fn check(&self, item: &WeightedIdentityRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityRecordEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedIdentityRecordRule for WeightedIdentityRecordEndsAtRangeRule {
    fn check(&self, item: &WeightedIdentityRecord) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a weighted identity record.
#[derive(Debug, Clone)]
pub struct WeightedIdentityRecordValidator {
    pub rules: Vec<Rc<dyn WeightedIdentityRecordRule>>,
}

impl WeightedIdentityRecordValidator {
    pub fn validate(&self, item: &WeightedIdentityRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedIdentityRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedIdentityRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_identity_record_validator() -> WeightedIdentityRecordValidator {
    WeightedIdentityRecordValidator {
        rules: vec![
            Rc::new(WeightedIdentityRecordLabelLengthRule { max_length: 32 }) as Rc<dyn WeightedIdentityRecordRule>,
            Rc::new(WeightedIdentityRecordRequiredLabelRule {
                message: "weighted identity record needs an entry".to_string(),
            }),
            Rc::new(WeightedIdentityRecordCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(WeightedIdentityRecordAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(WeightedIdentityRecordEndsAtRangeRule { min: 80, max: 255 }),
        ],
    }
}
