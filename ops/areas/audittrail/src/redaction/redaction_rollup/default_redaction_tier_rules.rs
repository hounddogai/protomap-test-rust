use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default redaction tier must pass. Check returns its problems, or none.
pub trait DefaultRedactionTierRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRedactionTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionTierRequiredExternalRefRule {
    pub message: String,
}

impl DefaultRedactionTierRule for DefaultRedactionTierRequiredExternalRefRule {
    fn check(&self, item: &DefaultRedactionTier) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionTierRequiredSourceRule {
    pub message: String,
}

impl DefaultRedactionTierRule for DefaultRedactionTierRequiredSourceRule {
    fn check(&self, item: &DefaultRedactionTier) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionTierDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultRedactionTierRule for DefaultRedactionTierDurationMinutesRangeRule {
    fn check(&self, item: &DefaultRedactionTier) -> Vec<String> {
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
pub struct DefaultRedactionTierRequiredNameRule {
    pub message: String,
}

impl DefaultRedactionTierRule for DefaultRedactionTierRequiredNameRule {
    fn check(&self, item: &DefaultRedactionTier) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionTierAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultRedactionTierRule for DefaultRedactionTierAttemptsRangeRule {
    fn check(&self, item: &DefaultRedactionTier) -> Vec<String> {
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
pub struct DefaultRedactionTierNameLengthRule {
    pub max_length: i64,
}

impl DefaultRedactionTierRule for DefaultRedactionTierNameLengthRule {
    fn check(&self, item: &DefaultRedactionTier) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default redaction tier.
#[derive(Debug, Clone)]
pub struct DefaultRedactionTierValidator {
    pub rules: Vec<Rc<dyn DefaultRedactionTierRule>>,
}

impl DefaultRedactionTierValidator {
    pub fn validate(&self, item: &DefaultRedactionTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultRedactionTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRedactionTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_redaction_tier_validator() -> DefaultRedactionTierValidator {
    DefaultRedactionTierValidator {
        rules: vec![
            Rc::new(DefaultRedactionTierRequiredExternalRefRule {
                message: "default redaction tier needs an entry".to_string(),
            }) as Rc<dyn DefaultRedactionTierRule>,
            Rc::new(DefaultRedactionTierRequiredSourceRule {
                message: "default redaction tier needs a value".to_string(),
            }),
            Rc::new(DefaultRedactionTierDurationMinutesRangeRule { min: 16, max: 80 }),
            Rc::new(DefaultRedactionTierRequiredNameRule {
                message: "default redaction tier needs this field".to_string(),
            }),
            Rc::new(DefaultRedactionTierAttemptsRangeRule { min: 32, max: 255 }),
            Rc::new(DefaultRedactionTierNameLengthRule { max_length: 32 }),
        ],
    }
}
