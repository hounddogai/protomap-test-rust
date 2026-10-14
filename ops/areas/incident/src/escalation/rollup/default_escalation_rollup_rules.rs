use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default escalation rollup must pass. Check returns its problems, or none.
pub trait DefaultEscalationRollupRule: std::fmt::Debug {
    fn check(&self, item: &DefaultEscalationRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEscalationRollupRequiredChecksumRule {
    pub message: String,
}

impl DefaultEscalationRollupRule for DefaultEscalationRollupRequiredChecksumRule {
    fn check(&self, item: &DefaultEscalationRollup) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEscalationRollupRequiredPhoneRule {
    pub message: String,
}

impl DefaultEscalationRollupRule for DefaultEscalationRollupRequiredPhoneRule {
    fn check(&self, item: &DefaultEscalationRollup) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEscalationRollupSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultEscalationRollupRule for DefaultEscalationRollupSequenceRangeRule {
    fn check(&self, item: &DefaultEscalationRollup) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEscalationRollupRequiredNameRule {
    pub message: String,
}

impl DefaultEscalationRollupRule for DefaultEscalationRollupRequiredNameRule {
    fn check(&self, item: &DefaultEscalationRollup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEscalationRollupChecksumLengthRule {
    pub max_length: i64,
}

impl DefaultEscalationRollupRule for DefaultEscalationRollupChecksumLengthRule {
    fn check(&self, item: &DefaultEscalationRollup) -> Vec<String> {
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

/// Runs every rule of a default escalation rollup.
#[derive(Debug, Clone)]
pub struct DefaultEscalationRollupValidator {
    pub rules: Vec<Rc<dyn DefaultEscalationRollupRule>>,
}

impl DefaultEscalationRollupValidator {
    pub fn validate(&self, item: &DefaultEscalationRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultEscalationRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultEscalationRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_escalation_rollup_validator() -> DefaultEscalationRollupValidator {
    DefaultEscalationRollupValidator {
        rules: vec![
            Rc::new(DefaultEscalationRollupRequiredChecksumRule {
                message: "default escalation rollup needs this field".to_string(),
            }) as Rc<dyn DefaultEscalationRollupRule>,
            Rc::new(DefaultEscalationRollupRequiredPhoneRule {
                message: "default escalation rollup needs an entry".to_string(),
            }),
            Rc::new(DefaultEscalationRollupSequenceRangeRule { min: 120, max: 1000 }),
            Rc::new(DefaultEscalationRollupRequiredNameRule {
                message: "default escalation rollup needs an entry".to_string(),
            }),
            Rc::new(DefaultEscalationRollupChecksumLengthRule { max_length: 255 }),
        ],
    }
}
