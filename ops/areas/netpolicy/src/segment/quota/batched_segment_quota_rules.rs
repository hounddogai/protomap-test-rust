use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched segment quota must pass. Check returns its problems, or none.
pub trait BatchedSegmentQuotaRule: std::fmt::Debug {
    fn check(&self, item: &BatchedSegmentQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentQuotaCodePrefixRule {
    pub prefix: String,
}

impl BatchedSegmentQuotaRule for BatchedSegmentQuotaCodePrefixRule {
    fn check(&self, item: &BatchedSegmentQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentQuotaRequiredDescriptionRule {
    pub message: String,
}

impl BatchedSegmentQuotaRule for BatchedSegmentQuotaRequiredDescriptionRule {
    fn check(&self, item: &BatchedSegmentQuota) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentQuotaRequiredCodeRule {
    pub message: String,
}

impl BatchedSegmentQuotaRule for BatchedSegmentQuotaRequiredCodeRule {
    fn check(&self, item: &BatchedSegmentQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched segment quota.
#[derive(Debug, Clone)]
pub struct BatchedSegmentQuotaValidator {
    pub rules: Vec<Rc<dyn BatchedSegmentQuotaRule>>,
}

impl BatchedSegmentQuotaValidator {
    pub fn validate(&self, item: &BatchedSegmentQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedSegmentQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedSegmentQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_segment_quota_validator() -> BatchedSegmentQuotaValidator {
    BatchedSegmentQuotaValidator {
        rules: vec![
            Rc::new(BatchedSegmentQuotaCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn BatchedSegmentQuotaRule>,
            Rc::new(BatchedSegmentQuotaRequiredDescriptionRule {
                message: "batched segment quota needs an entry".to_string(),
            }),
            Rc::new(BatchedSegmentQuotaRequiredCodeRule {
                message: "batched segment quota needs this field".to_string(),
            }),
        ],
    }
}
