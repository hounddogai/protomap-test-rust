use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a approval digest must pass. Check returns its problems, or none.
pub trait ApprovalDigestRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalDigestAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl ApprovalDigestRule for ApprovalDigestAllowedExternalRefRule {
    fn check(&self, item: &ApprovalDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is blocked", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalDigestAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalDigestRule for ApprovalDigestAttemptsRangeRule {
    fn check(&self, item: &ApprovalDigest) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalDigestTitleLengthRule {
    pub max_length: i64,
}

impl ApprovalDigestRule for ApprovalDigestTitleLengthRule {
    fn check(&self, item: &ApprovalDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval digest.
#[derive(Debug, Clone)]
pub struct ApprovalDigestValidator {
    pub rules: Vec<Rc<dyn ApprovalDigestRule>>,
}

impl ApprovalDigestValidator {
    pub fn validate(&self, item: &ApprovalDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ApprovalDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_digest_validator() -> ApprovalDigestValidator {
    ApprovalDigestValidator {
        rules: vec![
            Rc::new(ApprovalDigestAllowedExternalRefRule {
                allowed: vec!["default".to_string(), "manual".to_string()],
            }) as Rc<dyn ApprovalDigestRule>,
            Rc::new(ApprovalDigestAttemptsRangeRule { min: 255, max: 86400 }),
            Rc::new(ApprovalDigestTitleLengthRule { max_length: 64 }),
        ],
    }
}
