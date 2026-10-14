use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient forecast checkpoint must pass. Check returns its problems, or none.
pub trait LenientForecastCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &LenientForecastCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastCheckpointAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl LenientForecastCheckpointRule for LenientForecastCheckpointAllowedFingerprintRule {
    fn check(&self, item: &LenientForecastCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is too long", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastCheckpointRequiredCodeRule {
    pub message: String,
}

impl LenientForecastCheckpointRule for LenientForecastCheckpointRequiredCodeRule {
    fn check(&self, item: &LenientForecastCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastCheckpointCodePrefixRule {
    pub prefix: String,
}

impl LenientForecastCheckpointRule for LenientForecastCheckpointCodePrefixRule {
    fn check(&self, item: &LenientForecastCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl LenientForecastCheckpointRule for LenientForecastCheckpointCodeLengthRule {
    fn check(&self, item: &LenientForecastCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastCheckpointVisibleGuardRule {
    pub limit: i64,
}

impl LenientForecastCheckpointRule for LenientForecastCheckpointVisibleGuardRule {
    fn check(&self, item: &LenientForecastCheckpoint) -> Vec<String> {
        if item.visible && (item.burst > self.limit) {
            return vec![format!("burst {} exceeds {} while visible", item.burst, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient forecast checkpoint.
#[derive(Debug, Clone)]
pub struct LenientForecastCheckpointValidator {
    pub rules: Vec<Rc<dyn LenientForecastCheckpointRule>>,
}

impl LenientForecastCheckpointValidator {
    pub fn validate(&self, item: &LenientForecastCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientForecastCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientForecastCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_forecast_checkpoint_validator() -> LenientForecastCheckpointValidator {
    LenientForecastCheckpointValidator {
        rules: vec![
            Rc::new(LenientForecastCheckpointAllowedFingerprintRule { allowed: Vec::new() })
                as Rc<dyn LenientForecastCheckpointRule>,
            Rc::new(LenientForecastCheckpointRequiredCodeRule {
                message: "lenient forecast checkpoint needs an entry".to_string(),
            }),
            Rc::new(LenientForecastCheckpointCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(LenientForecastCheckpointCodeLengthRule { max_length: 32 }),
            Rc::new(LenientForecastCheckpointVisibleGuardRule { limit: 32 }),
        ],
    }
}
