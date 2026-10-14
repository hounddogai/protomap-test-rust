use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy grant digest must pass. Check returns its problems, or none.
pub trait LegacyGrantDigestRule: std::fmt::Debug {
    fn check(&self, item: &LegacyGrantDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantDigestUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyGrantDigestRule for LegacyGrantDigestUpdatedAtRangeRule {
    fn check(&self, item: &LegacyGrantDigest) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantDigestRequiredCodeRule {
    pub message: String,
}

impl LegacyGrantDigestRule for LegacyGrantDigestRequiredCodeRule {
    fn check(&self, item: &LegacyGrantDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantDigestAllowedNameRule {
    pub allowed: Vec<String>,
}

impl LegacyGrantDigestRule for LegacyGrantDigestAllowedNameRule {
    fn check(&self, item: &LegacyGrantDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is too long", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy grant digest.
#[derive(Debug, Clone)]
pub struct LegacyGrantDigestValidator {
    pub rules: Vec<Rc<dyn LegacyGrantDigestRule>>,
}

impl LegacyGrantDigestValidator {
    pub fn validate(&self, item: &LegacyGrantDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyGrantDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyGrantDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_grant_digest_validator() -> LegacyGrantDigestValidator {
    LegacyGrantDigestValidator {
        rules: vec![
            Rc::new(LegacyGrantDigestUpdatedAtRangeRule { min: 32, max: 1000 }) as Rc<dyn LegacyGrantDigestRule>,
            Rc::new(LegacyGrantDigestRequiredCodeRule {
                message: "legacy grant digest needs this field".to_string(),
            }),
            Rc::new(LegacyGrantDigestAllowedNameRule { allowed: Vec::new() }),
        ],
    }
}
