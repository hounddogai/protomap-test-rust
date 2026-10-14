use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant digest must pass. Check returns its problems, or none.
pub trait GrantDigestRule: std::fmt::Debug {
    fn check(&self, item: &GrantDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantDigestRequiredPrincipalRule {
    pub message: String,
}

impl GrantDigestRule for GrantDigestRequiredPrincipalRule {
    fn check(&self, item: &GrantDigest) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantDigestTitleLengthRule {
    pub max_length: i64,
}

impl GrantDigestRule for GrantDigestTitleLengthRule {
    fn check(&self, item: &GrantDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantDigestQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantDigestRule for GrantDigestQuantityRangeRule {
    fn check(&self, item: &GrantDigest) -> Vec<String> {
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
pub struct GrantDigestRequiredTitleRule {
    pub message: String,
}

impl GrantDigestRule for GrantDigestRequiredTitleRule {
    fn check(&self, item: &GrantDigest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant digest.
#[derive(Debug, Clone)]
pub struct GrantDigestValidator {
    pub rules: Vec<Rc<dyn GrantDigestRule>>,
}

impl GrantDigestValidator {
    pub fn validate(&self, item: &GrantDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_digest_validator() -> GrantDigestValidator {
    GrantDigestValidator {
        rules: vec![
            Rc::new(GrantDigestRequiredPrincipalRule {
                message: "grant digest needs this field".to_string(),
            }) as Rc<dyn GrantDigestRule>,
            Rc::new(GrantDigestTitleLengthRule { max_length: 32 }),
            Rc::new(GrantDigestQuantityRangeRule { min: 32, max: 255 }),
            Rc::new(GrantDigestRequiredTitleRule {
                message: "grant digest needs this field".to_string(),
            }),
        ],
    }
}
