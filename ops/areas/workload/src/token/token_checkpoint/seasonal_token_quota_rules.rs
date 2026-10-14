use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal token quota must pass. Check returns its problems, or none.
pub trait SeasonalTokenQuotaRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalTokenQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenQuotaRequiredIssuerRule {
    pub message: String,
}

impl SeasonalTokenQuotaRule for SeasonalTokenQuotaRequiredIssuerRule {
    fn check(&self, item: &SeasonalTokenQuota) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenQuotaAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalTokenQuotaRule for SeasonalTokenQuotaAttemptsRangeRule {
    fn check(&self, item: &SeasonalTokenQuota) -> Vec<String> {
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
pub struct SeasonalTokenQuotaCodePrefixRule {
    pub prefix: String,
}

impl SeasonalTokenQuotaRule for SeasonalTokenQuotaCodePrefixRule {
    fn check(&self, item: &SeasonalTokenQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenQuotaNameLengthRule {
    pub max_length: i64,
}

impl SeasonalTokenQuotaRule for SeasonalTokenQuotaNameLengthRule {
    fn check(&self, item: &SeasonalTokenQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenQuotaRequiredCodeRule {
    pub message: String,
}

impl SeasonalTokenQuotaRule for SeasonalTokenQuotaRequiredCodeRule {
    fn check(&self, item: &SeasonalTokenQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal token quota.
#[derive(Debug, Clone)]
pub struct SeasonalTokenQuotaValidator {
    pub rules: Vec<Rc<dyn SeasonalTokenQuotaRule>>,
}

impl SeasonalTokenQuotaValidator {
    pub fn validate(&self, item: &SeasonalTokenQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalTokenQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalTokenQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_token_quota_validator() -> SeasonalTokenQuotaValidator {
    SeasonalTokenQuotaValidator {
        rules: vec![
            Rc::new(SeasonalTokenQuotaRequiredIssuerRule {
                message: "seasonal token quota needs this field".to_string(),
            }) as Rc<dyn SeasonalTokenQuotaRule>,
            Rc::new(SeasonalTokenQuotaAttemptsRangeRule { min: 120, max: 3600 }),
            Rc::new(SeasonalTokenQuotaCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(SeasonalTokenQuotaNameLengthRule { max_length: 80 }),
            Rc::new(SeasonalTokenQuotaRequiredCodeRule {
                message: "seasonal token quota needs an entry".to_string(),
            }),
        ],
    }
}
