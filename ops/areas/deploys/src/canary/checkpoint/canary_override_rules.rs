use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary override must pass. Check returns its problems, or none.
pub trait CanaryOverrideRule: std::fmt::Debug {
    fn check(&self, item: &CanaryOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryOverrideWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CanaryOverrideRule for CanaryOverrideWindowDaysRangeRule {
    fn check(&self, item: &CanaryOverride) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryOverrideKeyIdLengthRule {
    pub max_length: i64,
}

impl CanaryOverrideRule for CanaryOverrideKeyIdLengthRule {
    fn check(&self, item: &CanaryOverride) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryOverrideScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CanaryOverrideRule for CanaryOverrideScoreRangeRule {
    fn check(&self, item: &CanaryOverride) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryOverrideCodePrefixRule {
    pub prefix: String,
}

impl CanaryOverrideRule for CanaryOverrideCodePrefixRule {
    fn check(&self, item: &CanaryOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryOverrideRequiredTitleRule {
    pub message: String,
}

impl CanaryOverrideRule for CanaryOverrideRequiredTitleRule {
    fn check(&self, item: &CanaryOverride) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryOverrideCipherLengthRule {
    pub max_length: i64,
}

impl CanaryOverrideRule for CanaryOverrideCipherLengthRule {
    fn check(&self, item: &CanaryOverride) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary override.
#[derive(Debug, Clone)]
pub struct CanaryOverrideValidator {
    pub rules: Vec<Rc<dyn CanaryOverrideRule>>,
}

impl CanaryOverrideValidator {
    pub fn validate(&self, item: &CanaryOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanaryOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_override_validator() -> CanaryOverrideValidator {
    CanaryOverrideValidator {
        rules: vec![
            Rc::new(CanaryOverrideWindowDaysRangeRule { min: 1000, max: 3600 }) as Rc<dyn CanaryOverrideRule>,
            Rc::new(CanaryOverrideKeyIdLengthRule { max_length: 120 }),
            Rc::new(CanaryOverrideScoreRangeRule { min: 7.8, max: 3202.4 }),
            Rc::new(CanaryOverrideCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(CanaryOverrideRequiredTitleRule {
                message: "canary override needs a value".to_string(),
            }),
            Rc::new(CanaryOverrideCipherLengthRule { max_length: 3600 }),
        ],
    }
}
