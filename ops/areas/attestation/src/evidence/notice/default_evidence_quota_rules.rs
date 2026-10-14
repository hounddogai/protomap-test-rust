use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default evidence quota must pass. Check returns its problems, or none.
pub trait DefaultEvidenceQuotaRule: std::fmt::Debug {
    fn check(&self, item: &DefaultEvidenceQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEvidenceQuotaThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultEvidenceQuotaRule for DefaultEvidenceQuotaThresholdRangeRule {
    fn check(&self, item: &DefaultEvidenceQuota) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEvidenceQuotaCodePrefixRule {
    pub prefix: String,
}

impl DefaultEvidenceQuotaRule for DefaultEvidenceQuotaCodePrefixRule {
    fn check(&self, item: &DefaultEvidenceQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEvidenceQuotaTitleLengthRule {
    pub max_length: i64,
}

impl DefaultEvidenceQuotaRule for DefaultEvidenceQuotaTitleLengthRule {
    fn check(&self, item: &DefaultEvidenceQuota) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEvidenceQuotaRequiredTitleRule {
    pub message: String,
}

impl DefaultEvidenceQuotaRule for DefaultEvidenceQuotaRequiredTitleRule {
    fn check(&self, item: &DefaultEvidenceQuota) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEvidenceQuotaRequiredAudienceRule {
    pub message: String,
}

impl DefaultEvidenceQuotaRule for DefaultEvidenceQuotaRequiredAudienceRule {
    fn check(&self, item: &DefaultEvidenceQuota) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEvidenceQuotaTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl DefaultEvidenceQuotaRule for DefaultEvidenceQuotaTagsLimitRule {
    fn check(&self, item: &DefaultEvidenceQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a default evidence quota.
#[derive(Debug, Clone)]
pub struct DefaultEvidenceQuotaValidator {
    pub rules: Vec<Rc<dyn DefaultEvidenceQuotaRule>>,
}

impl DefaultEvidenceQuotaValidator {
    pub fn validate(&self, item: &DefaultEvidenceQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultEvidenceQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultEvidenceQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_evidence_quota_validator() -> DefaultEvidenceQuotaValidator {
    DefaultEvidenceQuotaValidator {
        rules: vec![
            Rc::new(DefaultEvidenceQuotaThresholdRangeRule { min: 6.9, max: 4225.0 })
                as Rc<dyn DefaultEvidenceQuotaRule>,
            Rc::new(DefaultEvidenceQuotaCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(DefaultEvidenceQuotaTitleLengthRule { max_length: 86400 }),
            Rc::new(DefaultEvidenceQuotaRequiredTitleRule {
                message: "default evidence quota needs an entry".to_string(),
            }),
            Rc::new(DefaultEvidenceQuotaRequiredAudienceRule {
                message: "default evidence quota needs an entry".to_string(),
            }),
            Rc::new(DefaultEvidenceQuotaTagsLimitRule {
                max_count: 255,
                banned: vec!["adaptive".to_string()],
            }),
        ],
    }
}
