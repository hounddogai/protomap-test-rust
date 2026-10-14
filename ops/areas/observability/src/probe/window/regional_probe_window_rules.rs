use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional probe window must pass. Check returns its problems, or none.
pub trait RegionalProbeWindowRule: std::fmt::Debug {
    fn check(&self, item: &RegionalProbeWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeWindowRequiredNameRule {
    pub message: String,
}

impl RegionalProbeWindowRule for RegionalProbeWindowRequiredNameRule {
    fn check(&self, item: &RegionalProbeWindow) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeWindowActiveGuardRule {
    pub limit: f64,
}

impl RegionalProbeWindowRule for RegionalProbeWindowActiveGuardRule {
    fn check(&self, item: &RegionalProbeWindow) -> Vec<String> {
        if item.active && (item.ratio > self.limit) {
            return vec![format!("ratio {} exceeds {} while active", item.ratio, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeWindowRequiredCodeRule {
    pub message: String,
}

impl RegionalProbeWindowRule for RegionalProbeWindowRequiredCodeRule {
    fn check(&self, item: &RegionalProbeWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeWindowAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionalProbeWindowRule for RegionalProbeWindowAllowedCodeRule {
    fn check(&self, item: &RegionalProbeWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeWindowCodePrefixRule {
    pub prefix: String,
}

impl RegionalProbeWindowRule for RegionalProbeWindowCodePrefixRule {
    fn check(&self, item: &RegionalProbeWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional probe window.
#[derive(Debug, Clone)]
pub struct RegionalProbeWindowValidator {
    pub rules: Vec<Rc<dyn RegionalProbeWindowRule>>,
}

impl RegionalProbeWindowValidator {
    pub fn validate(&self, item: &RegionalProbeWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalProbeWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalProbeWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_probe_window_validator() -> RegionalProbeWindowValidator {
    RegionalProbeWindowValidator {
        rules: vec![
            Rc::new(RegionalProbeWindowRequiredNameRule {
                message: "regional probe window needs this field".to_string(),
            }) as Rc<dyn RegionalProbeWindowRule>,
            Rc::new(RegionalProbeWindowActiveGuardRule { limit: 7.0 }),
            Rc::new(RegionalProbeWindowRequiredCodeRule {
                message: "regional probe window needs a value".to_string(),
            }),
            Rc::new(RegionalProbeWindowAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "primary".to_string(), "tiered".to_string()],
            }),
            Rc::new(RegionalProbeWindowCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
