use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy nonce baseline must pass. Check returns its problems, or none.
pub trait LegacyNonceBaselineRule: std::fmt::Debug {
    fn check(&self, item: &LegacyNonceBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceBaselineAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl LegacyNonceBaselineRule for LegacyNonceBaselineAllowedUnitRule {
    fn check(&self, item: &LegacyNonceBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is expired", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceBaselineRequiredCodeRule {
    pub message: String,
}

impl LegacyNonceBaselineRule for LegacyNonceBaselineRequiredCodeRule {
    fn check(&self, item: &LegacyNonceBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceBaselineUnitLengthRule {
    pub max_length: i64,
}

impl LegacyNonceBaselineRule for LegacyNonceBaselineUnitLengthRule {
    fn check(&self, item: &LegacyNonceBaseline) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceBaselineFingerprintLengthRule {
    pub max_length: i64,
}

impl LegacyNonceBaselineRule for LegacyNonceBaselineFingerprintLengthRule {
    fn check(&self, item: &LegacyNonceBaseline) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceBaselineRequiredTitleRule {
    pub message: String,
}

impl LegacyNonceBaselineRule for LegacyNonceBaselineRequiredTitleRule {
    fn check(&self, item: &LegacyNonceBaseline) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceBaselinePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyNonceBaselineRule for LegacyNonceBaselinePriorityRangeRule {
    fn check(&self, item: &LegacyNonceBaseline) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy nonce baseline.
#[derive(Debug, Clone)]
pub struct LegacyNonceBaselineValidator {
    pub rules: Vec<Rc<dyn LegacyNonceBaselineRule>>,
}

impl LegacyNonceBaselineValidator {
    pub fn validate(&self, item: &LegacyNonceBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyNonceBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyNonceBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_nonce_baseline_validator() -> LegacyNonceBaselineValidator {
    LegacyNonceBaselineValidator {
        rules: vec![
            Rc::new(LegacyNonceBaselineAllowedUnitRule {
                allowed: vec!["tiered".to_string(), "rolling".to_string()],
            }) as Rc<dyn LegacyNonceBaselineRule>,
            Rc::new(LegacyNonceBaselineRequiredCodeRule {
                message: "legacy nonce baseline needs this field".to_string(),
            }),
            Rc::new(LegacyNonceBaselineUnitLengthRule { max_length: 255 }),
            Rc::new(LegacyNonceBaselineFingerprintLengthRule { max_length: 3600 }),
            Rc::new(LegacyNonceBaselineRequiredTitleRule {
                message: "legacy nonce baseline needs a value".to_string(),
            }),
            Rc::new(LegacyNonceBaselinePriorityRangeRule { min: 255, max: 1000 }),
        ],
    }
}
