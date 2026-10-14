use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule quota must pass. Check returns its problems, or none.
pub trait RuleQuotaRule: std::fmt::Debug {
    fn check(&self, item: &RuleQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleQuotaUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleQuotaRule for RuleQuotaUpdatedAtRangeRule {
    fn check(&self, item: &RuleQuota) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleQuotaCountryLengthRule {
    pub max_length: i64,
}

impl RuleQuotaRule for RuleQuotaCountryLengthRule {
    fn check(&self, item: &RuleQuota) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleQuotaDescriptionLengthRule {
    pub max_length: i64,
}

impl RuleQuotaRule for RuleQuotaDescriptionLengthRule {
    fn check(&self, item: &RuleQuota) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleQuotaRequiredCodeRule {
    pub message: String,
}

impl RuleQuotaRule for RuleQuotaRequiredCodeRule {
    fn check(&self, item: &RuleQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule quota.
#[derive(Debug, Clone)]
pub struct RuleQuotaValidator {
    pub rules: Vec<Rc<dyn RuleQuotaRule>>,
}

impl RuleQuotaValidator {
    pub fn validate(&self, item: &RuleQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_quota_validator() -> RuleQuotaValidator {
    RuleQuotaValidator {
        rules: vec![
            Rc::new(RuleQuotaUpdatedAtRangeRule { min: 16, max: 86400 }) as Rc<dyn RuleQuotaRule>,
            Rc::new(RuleQuotaCountryLengthRule { max_length: 255 }),
            Rc::new(RuleQuotaDescriptionLengthRule { max_length: 64 }),
            Rc::new(RuleQuotaRequiredCodeRule {
                message: "rule quota needs an entry".to_string(),
            }),
        ],
    }
}
