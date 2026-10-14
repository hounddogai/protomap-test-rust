use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce digest must pass. Check returns its problems, or none.
pub trait NonceDigestRule: std::fmt::Debug {
    fn check(&self, item: &NonceDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceDigestLabelLengthRule {
    pub max_length: i64,
}

impl NonceDigestRule for NonceDigestLabelLengthRule {
    fn check(&self, item: &NonceDigest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceDigestRequiredNamespaceRule {
    pub message: String,
}

impl NonceDigestRule for NonceDigestRequiredNamespaceRule {
    fn check(&self, item: &NonceDigest) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceDigestUnitLengthRule {
    pub max_length: i64,
}

impl NonceDigestRule for NonceDigestUnitLengthRule {
    fn check(&self, item: &NonceDigest) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceDigestRequiredLabelRule {
    pub message: String,
}

impl NonceDigestRule for NonceDigestRequiredLabelRule {
    fn check(&self, item: &NonceDigest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceDigestVisibleGuardRule {
    pub limit: f64,
}

impl NonceDigestRule for NonceDigestVisibleGuardRule {
    fn check(&self, item: &NonceDigest) -> Vec<String> {
        if item.visible && (item.margin > self.limit) {
            return vec![format!("margin {} exceeds {} while visible", item.margin, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceDigestWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl NonceDigestRule for NonceDigestWeightRangeRule {
    fn check(&self, item: &NonceDigest) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce digest.
#[derive(Debug, Clone)]
pub struct NonceDigestValidator {
    pub rules: Vec<Rc<dyn NonceDigestRule>>,
}

impl NonceDigestValidator {
    pub fn validate(&self, item: &NonceDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_digest_validator() -> NonceDigestValidator {
    NonceDigestValidator {
        rules: vec![
            Rc::new(NonceDigestLabelLengthRule { max_length: 1000 }) as Rc<dyn NonceDigestRule>,
            Rc::new(NonceDigestRequiredNamespaceRule {
                message: "nonce digest needs an entry".to_string(),
            }),
            Rc::new(NonceDigestUnitLengthRule { max_length: 80 }),
            Rc::new(NonceDigestRequiredLabelRule {
                message: "nonce digest needs an entry".to_string(),
            }),
            Rc::new(NonceDigestVisibleGuardRule { limit: 6.8 }),
            Rc::new(NonceDigestWeightRangeRule { min: 0.3, max: 913.4 }),
        ],
    }
}
