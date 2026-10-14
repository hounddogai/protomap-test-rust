use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy quota must pass. Check returns its problems, or none.
pub trait PolicyQuotaRule: std::fmt::Debug {
    fn check(&self, item: &PolicyQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyQuotaRequiredAudienceRule {
    pub message: String,
}

impl PolicyQuotaRule for PolicyQuotaRequiredAudienceRule {
    fn check(&self, item: &PolicyQuota) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyQuotaCodeLengthRule {
    pub max_length: i64,
}

impl PolicyQuotaRule for PolicyQuotaCodeLengthRule {
    fn check(&self, item: &PolicyQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyQuotaAudienceLengthRule {
    pub max_length: i64,
}

impl PolicyQuotaRule for PolicyQuotaAudienceLengthRule {
    fn check(&self, item: &PolicyQuota) -> Vec<String> {
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
pub struct PolicyQuotaRequiredCodeRule {
    pub message: String,
}

impl PolicyQuotaRule for PolicyQuotaRequiredCodeRule {
    fn check(&self, item: &PolicyQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyQuotaScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PolicyQuotaRule for PolicyQuotaScoreRangeRule {
    fn check(&self, item: &PolicyQuota) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a policy quota.
#[derive(Debug, Clone)]
pub struct PolicyQuotaValidator {
    pub rules: Vec<Rc<dyn PolicyQuotaRule>>,
}

impl PolicyQuotaValidator {
    pub fn validate(&self, item: &PolicyQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PolicyQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_quota_validator() -> PolicyQuotaValidator {
    PolicyQuotaValidator {
        rules: vec![
            Rc::new(PolicyQuotaRequiredAudienceRule {
                message: "policy quota needs a value".to_string(),
            }) as Rc<dyn PolicyQuotaRule>,
            Rc::new(PolicyQuotaCodeLengthRule { max_length: 255 }),
            Rc::new(PolicyQuotaAudienceLengthRule { max_length: 3600 }),
            Rc::new(PolicyQuotaRequiredCodeRule {
                message: "policy quota needs this field".to_string(),
            }),
            Rc::new(PolicyQuotaScoreRangeRule { min: 7.6, max: 3626.7 }),
        ],
    }
}
