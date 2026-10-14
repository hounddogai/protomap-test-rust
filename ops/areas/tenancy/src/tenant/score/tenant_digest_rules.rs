use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant digest must pass. Check returns its problems, or none.
pub trait TenantDigestRule: std::fmt::Debug {
    fn check(&self, item: &TenantDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantDigestCodePrefixRule {
    pub prefix: String,
}

impl TenantDigestRule for TenantDigestCodePrefixRule {
    fn check(&self, item: &TenantDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantDigestRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TenantDigestRule for TenantDigestRevisionRangeRule {
    fn check(&self, item: &TenantDigest) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantDigestCodeLengthRule {
    pub max_length: i64,
}

impl TenantDigestRule for TenantDigestCodeLengthRule {
    fn check(&self, item: &TenantDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant digest.
#[derive(Debug, Clone)]
pub struct TenantDigestValidator {
    pub rules: Vec<Rc<dyn TenantDigestRule>>,
}

impl TenantDigestValidator {
    pub fn validate(&self, item: &TenantDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_digest_validator() -> TenantDigestValidator {
    TenantDigestValidator {
        rules: vec![
            Rc::new(TenantDigestCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn TenantDigestRule>,
            Rc::new(TenantDigestRevisionRangeRule { min: 64, max: 80 }),
            Rc::new(TenantDigestCodeLengthRule { max_length: 32 }),
        ],
    }
}
