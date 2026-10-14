use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered silence target must pass. Check returns its problems, or none.
pub trait TieredSilenceTargetRule: std::fmt::Debug {
    fn check(&self, item: &TieredSilenceTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceTargetAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl TieredSilenceTargetRule for TieredSilenceTargetAllowedTitleRule {
    fn check(&self, item: &TieredSilenceTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is below minimum", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceTargetRequiredUnitRule {
    pub message: String,
}

impl TieredSilenceTargetRule for TieredSilenceTargetRequiredUnitRule {
    fn check(&self, item: &TieredSilenceTarget) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceTargetUnitLengthRule {
    pub max_length: i64,
}

impl TieredSilenceTargetRule for TieredSilenceTargetUnitLengthRule {
    fn check(&self, item: &TieredSilenceTarget) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceTargetDescriptionLengthRule {
    pub max_length: i64,
}

impl TieredSilenceTargetRule for TieredSilenceTargetDescriptionLengthRule {
    fn check(&self, item: &TieredSilenceTarget) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceTargetRequiredCodeRule {
    pub message: String,
}

impl TieredSilenceTargetRule for TieredSilenceTargetRequiredCodeRule {
    fn check(&self, item: &TieredSilenceTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered silence target.
#[derive(Debug, Clone)]
pub struct TieredSilenceTargetValidator {
    pub rules: Vec<Rc<dyn TieredSilenceTargetRule>>,
}

impl TieredSilenceTargetValidator {
    pub fn validate(&self, item: &TieredSilenceTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredSilenceTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredSilenceTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_silence_target_validator() -> TieredSilenceTargetValidator {
    TieredSilenceTargetValidator {
        rules: vec![
            Rc::new(TieredSilenceTargetAllowedTitleRule { allowed: Vec::new() }) as Rc<dyn TieredSilenceTargetRule>,
            Rc::new(TieredSilenceTargetRequiredUnitRule {
                message: "tiered silence target needs a value".to_string(),
            }),
            Rc::new(TieredSilenceTargetUnitLengthRule { max_length: 1000 }),
            Rc::new(TieredSilenceTargetDescriptionLengthRule { max_length: 1000 }),
            Rc::new(TieredSilenceTargetRequiredCodeRule {
                message: "tiered silence target needs a value".to_string(),
            }),
        ],
    }
}
