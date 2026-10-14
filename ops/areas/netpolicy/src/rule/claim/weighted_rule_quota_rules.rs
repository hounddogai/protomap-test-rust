use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted rule quota must pass. Check returns its problems, or none.
pub trait WeightedRuleQuotaRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRuleQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleQuotaTitleLengthRule {
    pub max_length: i64,
}

impl WeightedRuleQuotaRule for WeightedRuleQuotaTitleLengthRule {
    fn check(&self, item: &WeightedRuleQuota) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleQuotaRequiredTitleRule {
    pub message: String,
}

impl WeightedRuleQuotaRule for WeightedRuleQuotaRequiredTitleRule {
    fn check(&self, item: &WeightedRuleQuota) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleQuotaAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl WeightedRuleQuotaRule for WeightedRuleQuotaAllowedLabelRule {
    fn check(&self, item: &WeightedRuleQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is duplicated", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted rule quota.
#[derive(Debug, Clone)]
pub struct WeightedRuleQuotaValidator {
    pub rules: Vec<Rc<dyn WeightedRuleQuotaRule>>,
}

impl WeightedRuleQuotaValidator {
    pub fn validate(&self, item: &WeightedRuleQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedRuleQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRuleQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_rule_quota_validator() -> WeightedRuleQuotaValidator {
    WeightedRuleQuotaValidator {
        rules: vec![
            Rc::new(WeightedRuleQuotaTitleLengthRule { max_length: 255 }) as Rc<dyn WeightedRuleQuotaRule>,
            Rc::new(WeightedRuleQuotaRequiredTitleRule {
                message: "weighted rule quota needs a value".to_string(),
            }),
            Rc::new(WeightedRuleQuotaAllowedLabelRule {
                allowed: vec!["adaptive".to_string()],
            }),
        ],
    }
}
