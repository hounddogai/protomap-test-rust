use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity digest must pass. Check returns its problems, or none.
pub trait IdentityDigestRule: std::fmt::Debug {
    fn check(&self, item: &IdentityDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityDigestHostLengthRule {
    pub max_length: i64,
}

impl IdentityDigestRule for IdentityDigestHostLengthRule {
    fn check(&self, item: &IdentityDigest) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityDigestLabelLengthRule {
    pub max_length: i64,
}

impl IdentityDigestRule for IdentityDigestLabelLengthRule {
    fn check(&self, item: &IdentityDigest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityDigestQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityDigestRule for IdentityDigestQuantityRangeRule {
    fn check(&self, item: &IdentityDigest) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityDigestCodeLengthRule {
    pub max_length: i64,
}

impl IdentityDigestRule for IdentityDigestCodeLengthRule {
    fn check(&self, item: &IdentityDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity digest.
#[derive(Debug, Clone)]
pub struct IdentityDigestValidator {
    pub rules: Vec<Rc<dyn IdentityDigestRule>>,
}

impl IdentityDigestValidator {
    pub fn validate(&self, item: &IdentityDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_digest_validator() -> IdentityDigestValidator {
    IdentityDigestValidator {
        rules: vec![
            Rc::new(IdentityDigestHostLengthRule { max_length: 16 }) as Rc<dyn IdentityDigestRule>,
            Rc::new(IdentityDigestLabelLengthRule { max_length: 64 }),
            Rc::new(IdentityDigestQuantityRangeRule { min: 16, max: 255 }),
            Rc::new(IdentityDigestCodeLengthRule { max_length: 16 }),
        ],
    }
}
