use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release profile must pass. Check returns its problems, or none.
pub trait ReleaseProfileRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseProfileDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseProfileRule for ReleaseProfileDurationMinutesRangeRule {
    fn check(&self, item: &ReleaseProfile) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseProfileLabelLengthRule {
    pub max_length: i64,
}

impl ReleaseProfileRule for ReleaseProfileLabelLengthRule {
    fn check(&self, item: &ReleaseProfile) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseProfileRequiredLabelRule {
    pub message: String,
}

impl ReleaseProfileRule for ReleaseProfileRequiredLabelRule {
    fn check(&self, item: &ReleaseProfile) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseProfileRequiredUnitRule {
    pub message: String,
}

impl ReleaseProfileRule for ReleaseProfileRequiredUnitRule {
    fn check(&self, item: &ReleaseProfile) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseProfileAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseProfileRule for ReleaseProfileAttemptsRangeRule {
    fn check(&self, item: &ReleaseProfile) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseProfileCodePrefixRule {
    pub prefix: String,
}

impl ReleaseProfileRule for ReleaseProfileCodePrefixRule {
    fn check(&self, item: &ReleaseProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release profile.
#[derive(Debug, Clone)]
pub struct ReleaseProfileValidator {
    pub rules: Vec<Rc<dyn ReleaseProfileRule>>,
}

impl ReleaseProfileValidator {
    pub fn validate(&self, item: &ReleaseProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_profile_validator() -> ReleaseProfileValidator {
    ReleaseProfileValidator {
        rules: vec![
            Rc::new(ReleaseProfileDurationMinutesRangeRule { min: 32, max: 255 }) as Rc<dyn ReleaseProfileRule>,
            Rc::new(ReleaseProfileLabelLengthRule { max_length: 64 }),
            Rc::new(ReleaseProfileRequiredLabelRule {
                message: "release profile needs a value".to_string(),
            }),
            Rc::new(ReleaseProfileRequiredUnitRule {
                message: "release profile needs a value".to_string(),
            }),
            Rc::new(ReleaseProfileAttemptsRangeRule { min: 80, max: 3600 }),
            Rc::new(ReleaseProfileCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
