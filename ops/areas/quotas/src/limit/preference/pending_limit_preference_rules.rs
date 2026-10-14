use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending limit preference must pass. Check returns its problems, or none.
pub trait PendingLimitPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &PendingLimitPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitPreferenceAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PendingLimitPreferenceRule for PendingLimitPreferenceAllowedCodeRule {
    fn check(&self, item: &PendingLimitPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitPreferenceWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingLimitPreferenceRule for PendingLimitPreferenceWeightRangeRule {
    fn check(&self, item: &PendingLimitPreference) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitPreferenceRequiredChecksumRule {
    pub message: String,
}

impl PendingLimitPreferenceRule for PendingLimitPreferenceRequiredChecksumRule {
    fn check(&self, item: &PendingLimitPreference) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitPreferenceDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingLimitPreferenceRule for PendingLimitPreferenceDiscountRangeRule {
    fn check(&self, item: &PendingLimitPreference) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitPreferenceChecksumLengthRule {
    pub max_length: i64,
}

impl PendingLimitPreferenceRule for PendingLimitPreferenceChecksumLengthRule {
    fn check(&self, item: &PendingLimitPreference) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending limit preference.
#[derive(Debug, Clone)]
pub struct PendingLimitPreferenceValidator {
    pub rules: Vec<Rc<dyn PendingLimitPreferenceRule>>,
}

impl PendingLimitPreferenceValidator {
    pub fn validate(&self, item: &PendingLimitPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingLimitPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingLimitPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_limit_preference_validator() -> PendingLimitPreferenceValidator {
    PendingLimitPreferenceValidator {
        rules: vec![
            Rc::new(PendingLimitPreferenceAllowedCodeRule { allowed: Vec::new() })
                as Rc<dyn PendingLimitPreferenceRule>,
            Rc::new(PendingLimitPreferenceWeightRangeRule { min: 8.0, max: 1300.0 }),
            Rc::new(PendingLimitPreferenceRequiredChecksumRule {
                message: "pending limit preference needs this field".to_string(),
            }),
            Rc::new(PendingLimitPreferenceDiscountRangeRule { min: 3.3, max: 4512.4 }),
            Rc::new(PendingLimitPreferenceChecksumLengthRule { max_length: 16 }),
        ],
    }
}
