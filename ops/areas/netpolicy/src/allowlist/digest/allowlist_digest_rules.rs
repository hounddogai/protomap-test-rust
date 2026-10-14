use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a allowlist digest must pass. Check returns its problems, or none.
pub trait AllowlistDigestRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistDigestRequiredAudienceRule {
    pub message: String,
}

impl AllowlistDigestRule for AllowlistDigestRequiredAudienceRule {
    fn check(&self, item: &AllowlistDigest) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistDigestCodePrefixRule {
    pub prefix: String,
}

impl AllowlistDigestRule for AllowlistDigestCodePrefixRule {
    fn check(&self, item: &AllowlistDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistDigestRequiredCodeRule {
    pub message: String,
}

impl AllowlistDigestRule for AllowlistDigestRequiredCodeRule {
    fn check(&self, item: &AllowlistDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistDigestAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AllowlistDigestRule for AllowlistDigestAllowedCodeRule {
    fn check(&self, item: &AllowlistDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistDigestCodeLengthRule {
    pub max_length: i64,
}

impl AllowlistDigestRule for AllowlistDigestCodeLengthRule {
    fn check(&self, item: &AllowlistDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistDigestWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistDigestRule for AllowlistDigestWindowDaysRangeRule {
    fn check(&self, item: &AllowlistDigest) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a allowlist digest.
#[derive(Debug, Clone)]
pub struct AllowlistDigestValidator {
    pub rules: Vec<Rc<dyn AllowlistDigestRule>>,
}

impl AllowlistDigestValidator {
    pub fn validate(&self, item: &AllowlistDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AllowlistDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_digest_validator() -> AllowlistDigestValidator {
    AllowlistDigestValidator {
        rules: vec![
            Rc::new(AllowlistDigestRequiredAudienceRule {
                message: "allowlist digest needs a value".to_string(),
            }) as Rc<dyn AllowlistDigestRule>,
            Rc::new(AllowlistDigestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(AllowlistDigestRequiredCodeRule {
                message: "allowlist digest needs a value".to_string(),
            }),
            Rc::new(AllowlistDigestAllowedCodeRule {
                allowed: vec!["adaptive".to_string(), "primary".to_string(), "strict".to_string()],
            }),
            Rc::new(AllowlistDigestCodeLengthRule { max_length: 16 }),
            Rc::new(AllowlistDigestWindowDaysRangeRule { min: 32, max: 80 }),
        ],
    }
}
