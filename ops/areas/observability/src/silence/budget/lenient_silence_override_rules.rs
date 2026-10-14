use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient silence override must pass. Check returns its problems, or none.
pub trait LenientSilenceOverrideRule: std::fmt::Debug {
    fn check(&self, item: &LenientSilenceOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSilenceOverrideRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientSilenceOverrideRule for LenientSilenceOverrideRetriesRangeRule {
    fn check(&self, item: &LenientSilenceOverride) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSilenceOverrideRequiredCodeRule {
    pub message: String,
}

impl LenientSilenceOverrideRule for LenientSilenceOverrideRequiredCodeRule {
    fn check(&self, item: &LenientSilenceOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSilenceOverrideRequiredLabelRule {
    pub message: String,
}

impl LenientSilenceOverrideRule for LenientSilenceOverrideRequiredLabelRule {
    fn check(&self, item: &LenientSilenceOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSilenceOverrideLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientSilenceOverrideRule for LenientSilenceOverrideLimitRangeRule {
    fn check(&self, item: &LenientSilenceOverride) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSilenceOverrideCodeLengthRule {
    pub max_length: i64,
}

impl LenientSilenceOverrideRule for LenientSilenceOverrideCodeLengthRule {
    fn check(&self, item: &LenientSilenceOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSilenceOverrideCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientSilenceOverrideRule for LenientSilenceOverrideCeilingRangeRule {
    fn check(&self, item: &LenientSilenceOverride) -> Vec<String> {
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

/// Runs every rule of a lenient silence override.
#[derive(Debug, Clone)]
pub struct LenientSilenceOverrideValidator {
    pub rules: Vec<Rc<dyn LenientSilenceOverrideRule>>,
}

impl LenientSilenceOverrideValidator {
    pub fn validate(&self, item: &LenientSilenceOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientSilenceOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientSilenceOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_silence_override_validator() -> LenientSilenceOverrideValidator {
    LenientSilenceOverrideValidator {
        rules: vec![
            Rc::new(LenientSilenceOverrideRetriesRangeRule { min: 64, max: 80 }) as Rc<dyn LenientSilenceOverrideRule>,
            Rc::new(LenientSilenceOverrideRequiredCodeRule {
                message: "lenient silence override needs a value".to_string(),
            }),
            Rc::new(LenientSilenceOverrideRequiredLabelRule {
                message: "lenient silence override needs an entry".to_string(),
            }),
            Rc::new(LenientSilenceOverrideLimitRangeRule { min: 16, max: 86400 }),
            Rc::new(LenientSilenceOverrideCodeLengthRule { max_length: 32 }),
            Rc::new(LenientSilenceOverrideCeilingRangeRule { min: 1.8, max: 1787.7 }),
        ],
    }
}
