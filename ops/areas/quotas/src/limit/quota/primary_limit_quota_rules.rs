use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary limit quota must pass. Check returns its problems, or none.
pub trait PrimaryLimitQuotaRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryLimitQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitQuotaRequiredCodeRule {
    pub message: String,
}

impl PrimaryLimitQuotaRule for PrimaryLimitQuotaRequiredCodeRule {
    fn check(&self, item: &PrimaryLimitQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitQuotaCurrencyLengthRule {
    pub max_length: i64,
}

impl PrimaryLimitQuotaRule for PrimaryLimitQuotaCurrencyLengthRule {
    fn check(&self, item: &PrimaryLimitQuota) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitQuotaRequiredTitleRule {
    pub message: String,
}

impl PrimaryLimitQuotaRule for PrimaryLimitQuotaRequiredTitleRule {
    fn check(&self, item: &PrimaryLimitQuota) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitQuotaCodePrefixRule {
    pub prefix: String,
}

impl PrimaryLimitQuotaRule for PrimaryLimitQuotaCodePrefixRule {
    fn check(&self, item: &PrimaryLimitQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary limit quota.
#[derive(Debug, Clone)]
pub struct PrimaryLimitQuotaValidator {
    pub rules: Vec<Rc<dyn PrimaryLimitQuotaRule>>,
}

impl PrimaryLimitQuotaValidator {
    pub fn validate(&self, item: &PrimaryLimitQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryLimitQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryLimitQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_limit_quota_validator() -> PrimaryLimitQuotaValidator {
    PrimaryLimitQuotaValidator {
        rules: vec![
            Rc::new(PrimaryLimitQuotaRequiredCodeRule {
                message: "primary limit quota needs an entry".to_string(),
            }) as Rc<dyn PrimaryLimitQuotaRule>,
            Rc::new(PrimaryLimitQuotaCurrencyLengthRule { max_length: 32 }),
            Rc::new(PrimaryLimitQuotaRequiredTitleRule {
                message: "primary limit quota needs an entry".to_string(),
            }),
            Rc::new(PrimaryLimitQuotaCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
