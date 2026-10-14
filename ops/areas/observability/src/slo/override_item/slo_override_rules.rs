use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a slo override must pass. Check returns its problems, or none.
pub trait SloOverrideRule: std::fmt::Debug {
    fn check(&self, item: &SloOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloOverrideCodePrefixRule {
    pub prefix: String,
}

impl SloOverrideRule for SloOverrideCodePrefixRule {
    fn check(&self, item: &SloOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloOverrideAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SloOverrideRule for SloOverrideAllowedCodeRule {
    fn check(&self, item: &SloOverride) -> Vec<String> {
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
pub struct SloOverrideCodeLengthRule {
    pub max_length: i64,
}

impl SloOverrideRule for SloOverrideCodeLengthRule {
    fn check(&self, item: &SloOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloOverrideThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SloOverrideRule for SloOverrideThresholdRangeRule {
    fn check(&self, item: &SloOverride) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloOverrideRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloOverrideRule for SloOverrideRevisionRangeRule {
    fn check(&self, item: &SloOverride) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo override.
#[derive(Debug, Clone)]
pub struct SloOverrideValidator {
    pub rules: Vec<Rc<dyn SloOverrideRule>>,
}

impl SloOverrideValidator {
    pub fn validate(&self, item: &SloOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_override_validator() -> SloOverrideValidator {
    SloOverrideValidator {
        rules: vec![
            Rc::new(SloOverrideCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn SloOverrideRule>,
            Rc::new(SloOverrideAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(SloOverrideCodeLengthRule { max_length: 120 }),
            Rc::new(SloOverrideThresholdRangeRule { min: 0.4, max: 2232.1 }),
            Rc::new(SloOverrideRevisionRangeRule { min: 80, max: 120 }),
        ],
    }
}
