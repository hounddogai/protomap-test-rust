use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a usage digest must pass. Check returns its problems, or none.
pub trait UsageDigestRule: std::fmt::Debug {
    fn check(&self, item: &UsageDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageDigestLabelLengthRule {
    pub max_length: i64,
}

impl UsageDigestRule for UsageDigestLabelLengthRule {
    fn check(&self, item: &UsageDigest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageDigestRequiredScopeRule {
    pub message: String,
}

impl UsageDigestRule for UsageDigestRequiredScopeRule {
    fn check(&self, item: &UsageDigest) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageDigestTenantIdLengthRule {
    pub max_length: i64,
}

impl UsageDigestRule for UsageDigestTenantIdLengthRule {
    fn check(&self, item: &UsageDigest) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageDigestCodeLengthRule {
    pub max_length: i64,
}

impl UsageDigestRule for UsageDigestCodeLengthRule {
    fn check(&self, item: &UsageDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage digest.
#[derive(Debug, Clone)]
pub struct UsageDigestValidator {
    pub rules: Vec<Rc<dyn UsageDigestRule>>,
}

impl UsageDigestValidator {
    pub fn validate(&self, item: &UsageDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &UsageDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_digest_validator() -> UsageDigestValidator {
    UsageDigestValidator {
        rules: vec![
            Rc::new(UsageDigestLabelLengthRule { max_length: 120 }) as Rc<dyn UsageDigestRule>,
            Rc::new(UsageDigestRequiredScopeRule {
                message: "usage digest needs a value".to_string(),
            }),
            Rc::new(UsageDigestTenantIdLengthRule { max_length: 86400 }),
            Rc::new(UsageDigestCodeLengthRule { max_length: 64 }),
        ],
    }
}
