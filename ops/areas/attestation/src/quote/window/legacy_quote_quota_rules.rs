use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy quote quota must pass. Check returns its problems, or none.
pub trait LegacyQuoteQuotaRule: std::fmt::Debug {
    fn check(&self, item: &LegacyQuoteQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteQuotaAudienceLengthRule {
    pub max_length: i64,
}

impl LegacyQuoteQuotaRule for LegacyQuoteQuotaAudienceLengthRule {
    fn check(&self, item: &LegacyQuoteQuota) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteQuotaRequiredCodeRule {
    pub message: String,
}

impl LegacyQuoteQuotaRule for LegacyQuoteQuotaRequiredCodeRule {
    fn check(&self, item: &LegacyQuoteQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteQuotaStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyQuoteQuotaRule for LegacyQuoteQuotaStartsAtRangeRule {
    fn check(&self, item: &LegacyQuoteQuota) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteQuotaCodePrefixRule {
    pub prefix: String,
}

impl LegacyQuoteQuotaRule for LegacyQuoteQuotaCodePrefixRule {
    fn check(&self, item: &LegacyQuoteQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteQuotaEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyQuoteQuotaRule for LegacyQuoteQuotaEndsAtRangeRule {
    fn check(&self, item: &LegacyQuoteQuota) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteQuotaRequiredTitleRule {
    pub message: String,
}

impl LegacyQuoteQuotaRule for LegacyQuoteQuotaRequiredTitleRule {
    fn check(&self, item: &LegacyQuoteQuota) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy quote quota.
#[derive(Debug, Clone)]
pub struct LegacyQuoteQuotaValidator {
    pub rules: Vec<Rc<dyn LegacyQuoteQuotaRule>>,
}

impl LegacyQuoteQuotaValidator {
    pub fn validate(&self, item: &LegacyQuoteQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyQuoteQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyQuoteQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_quote_quota_validator() -> LegacyQuoteQuotaValidator {
    LegacyQuoteQuotaValidator {
        rules: vec![
            Rc::new(LegacyQuoteQuotaAudienceLengthRule { max_length: 80 }) as Rc<dyn LegacyQuoteQuotaRule>,
            Rc::new(LegacyQuoteQuotaRequiredCodeRule {
                message: "legacy quote quota needs an entry".to_string(),
            }),
            Rc::new(LegacyQuoteQuotaStartsAtRangeRule { min: 32, max: 80 }),
            Rc::new(LegacyQuoteQuotaCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(LegacyQuoteQuotaEndsAtRangeRule { min: 16, max: 32 }),
            Rc::new(LegacyQuoteQuotaRequiredTitleRule {
                message: "legacy quote quota needs an entry".to_string(),
            }),
        ],
    }
}
