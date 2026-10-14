use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered token bundle must pass. Check returns its problems, or none.
pub trait TieredTokenBundleRule: std::fmt::Debug {
    fn check(&self, item: &TieredTokenBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenBundleCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredTokenBundleRule for TieredTokenBundleCapacityRangeRule {
    fn check(&self, item: &TieredTokenBundle) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenBundleNameLengthRule {
    pub max_length: i64,
}

impl TieredTokenBundleRule for TieredTokenBundleNameLengthRule {
    fn check(&self, item: &TieredTokenBundle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenBundleCodePrefixRule {
    pub prefix: String,
}

impl TieredTokenBundleRule for TieredTokenBundleCodePrefixRule {
    fn check(&self, item: &TieredTokenBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenBundleReviewerLengthRule {
    pub max_length: i64,
}

impl TieredTokenBundleRule for TieredTokenBundleReviewerLengthRule {
    fn check(&self, item: &TieredTokenBundle) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenBundleAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl TieredTokenBundleRule for TieredTokenBundleAllowedReviewerRule {
    fn check(&self, item: &TieredTokenBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is duplicated", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenBundleCodeLengthRule {
    pub max_length: i64,
}

impl TieredTokenBundleRule for TieredTokenBundleCodeLengthRule {
    fn check(&self, item: &TieredTokenBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered token bundle.
#[derive(Debug, Clone)]
pub struct TieredTokenBundleValidator {
    pub rules: Vec<Rc<dyn TieredTokenBundleRule>>,
}

impl TieredTokenBundleValidator {
    pub fn validate(&self, item: &TieredTokenBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredTokenBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredTokenBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_token_bundle_validator() -> TieredTokenBundleValidator {
    TieredTokenBundleValidator {
        rules: vec![
            Rc::new(TieredTokenBundleCapacityRangeRule { min: 120, max: 255 }) as Rc<dyn TieredTokenBundleRule>,
            Rc::new(TieredTokenBundleNameLengthRule { max_length: 16 }),
            Rc::new(TieredTokenBundleCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(TieredTokenBundleReviewerLengthRule { max_length: 255 }),
            Rc::new(TieredTokenBundleAllowedReviewerRule {
                allowed: vec!["regional".to_string(), "primary".to_string(), "tiered".to_string()],
            }),
            Rc::new(TieredTokenBundleCodeLengthRule { max_length: 32 }),
        ],
    }
}
