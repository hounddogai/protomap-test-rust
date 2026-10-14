use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional grant quota must pass. Check returns its problems, or none.
pub trait RegionalGrantQuotaRule: std::fmt::Debug {
    fn check(&self, item: &RegionalGrantQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalGrantQuotaNameLengthRule {
    pub max_length: i64,
}

impl RegionalGrantQuotaRule for RegionalGrantQuotaNameLengthRule {
    fn check(&self, item: &RegionalGrantQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalGrantQuotaCodePrefixRule {
    pub prefix: String,
}

impl RegionalGrantQuotaRule for RegionalGrantQuotaCodePrefixRule {
    fn check(&self, item: &RegionalGrantQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalGrantQuotaRequiredCodeRule {
    pub message: String,
}

impl RegionalGrantQuotaRule for RegionalGrantQuotaRequiredCodeRule {
    fn check(&self, item: &RegionalGrantQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional grant quota.
#[derive(Debug, Clone)]
pub struct RegionalGrantQuotaValidator {
    pub rules: Vec<Rc<dyn RegionalGrantQuotaRule>>,
}

impl RegionalGrantQuotaValidator {
    pub fn validate(&self, item: &RegionalGrantQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalGrantQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalGrantQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_grant_quota_validator() -> RegionalGrantQuotaValidator {
    RegionalGrantQuotaValidator {
        rules: vec![
            Rc::new(RegionalGrantQuotaNameLengthRule { max_length: 120 }) as Rc<dyn RegionalGrantQuotaRule>,
            Rc::new(RegionalGrantQuotaCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RegionalGrantQuotaRequiredCodeRule {
                message: "regional grant quota needs this field".to_string(),
            }),
        ],
    }
}
