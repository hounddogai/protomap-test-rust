use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling token digest must pass. Check returns its problems, or none.
pub trait RollingTokenDigestRule: std::fmt::Debug {
    fn check(&self, item: &RollingTokenDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenDigestUnitLengthRule {
    pub max_length: i64,
}

impl RollingTokenDigestRule for RollingTokenDigestUnitLengthRule {
    fn check(&self, item: &RollingTokenDigest) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenDigestAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RollingTokenDigestRule for RollingTokenDigestAllowedTitleRule {
    fn check(&self, item: &RollingTokenDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is above maximum", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenDigestRequiredTitleRule {
    pub message: String,
}

impl RollingTokenDigestRule for RollingTokenDigestRequiredTitleRule {
    fn check(&self, item: &RollingTokenDigest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenDigestCodeLengthRule {
    pub max_length: i64,
}

impl RollingTokenDigestRule for RollingTokenDigestCodeLengthRule {
    fn check(&self, item: &RollingTokenDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenDigestTitleLengthRule {
    pub max_length: i64,
}

impl RollingTokenDigestRule for RollingTokenDigestTitleLengthRule {
    fn check(&self, item: &RollingTokenDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenDigestCodePrefixRule {
    pub prefix: String,
}

impl RollingTokenDigestRule for RollingTokenDigestCodePrefixRule {
    fn check(&self, item: &RollingTokenDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling token digest.
#[derive(Debug, Clone)]
pub struct RollingTokenDigestValidator {
    pub rules: Vec<Rc<dyn RollingTokenDigestRule>>,
}

impl RollingTokenDigestValidator {
    pub fn validate(&self, item: &RollingTokenDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingTokenDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingTokenDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_token_digest_validator() -> RollingTokenDigestValidator {
    RollingTokenDigestValidator {
        rules: vec![
            Rc::new(RollingTokenDigestUnitLengthRule { max_length: 120 }) as Rc<dyn RollingTokenDigestRule>,
            Rc::new(RollingTokenDigestAllowedTitleRule {
                allowed: vec!["rolling".to_string()],
            }),
            Rc::new(RollingTokenDigestRequiredTitleRule {
                message: "rolling token digest needs this field".to_string(),
            }),
            Rc::new(RollingTokenDigestCodeLengthRule { max_length: 64 }),
            Rc::new(RollingTokenDigestTitleLengthRule { max_length: 86400 }),
            Rc::new(RollingTokenDigestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
