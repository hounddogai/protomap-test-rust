use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default silence override must pass. Check returns its problems, or none.
pub trait DefaultSilenceOverrideRule: std::fmt::Debug {
    fn check(&self, item: &DefaultSilenceOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSilenceOverrideCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultSilenceOverrideRule for DefaultSilenceOverrideCreatedAtRangeRule {
    fn check(&self, item: &DefaultSilenceOverride) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSilenceOverridePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultSilenceOverrideRule for DefaultSilenceOverridePortRangeRule {
    fn check(&self, item: &DefaultSilenceOverride) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSilenceOverrideTitleLengthRule {
    pub max_length: i64,
}

impl DefaultSilenceOverrideRule for DefaultSilenceOverrideTitleLengthRule {
    fn check(&self, item: &DefaultSilenceOverride) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSilenceOverrideRequiredCodeRule {
    pub message: String,
}

impl DefaultSilenceOverrideRule for DefaultSilenceOverrideRequiredCodeRule {
    fn check(&self, item: &DefaultSilenceOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default silence override.
#[derive(Debug, Clone)]
pub struct DefaultSilenceOverrideValidator {
    pub rules: Vec<Rc<dyn DefaultSilenceOverrideRule>>,
}

impl DefaultSilenceOverrideValidator {
    pub fn validate(&self, item: &DefaultSilenceOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultSilenceOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultSilenceOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_silence_override_validator() -> DefaultSilenceOverrideValidator {
    DefaultSilenceOverrideValidator {
        rules: vec![
            Rc::new(DefaultSilenceOverrideCreatedAtRangeRule { min: 32, max: 1000 })
                as Rc<dyn DefaultSilenceOverrideRule>,
            Rc::new(DefaultSilenceOverridePortRangeRule { min: 1000, max: 1000 }),
            Rc::new(DefaultSilenceOverrideTitleLengthRule { max_length: 120 }),
            Rc::new(DefaultSilenceOverrideRequiredCodeRule {
                message: "default silence override needs a value".to_string(),
            }),
        ],
    }
}
