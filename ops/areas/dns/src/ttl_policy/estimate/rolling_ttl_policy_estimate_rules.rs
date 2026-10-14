use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling ttl policy estimate must pass. Check returns its problems, or none.
pub trait RollingTtlPolicyEstimateRule: std::fmt::Debug {
    fn check(&self, item: &RollingTtlPolicyEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyEstimateNamespaceLengthRule {
    pub max_length: i64,
}

impl RollingTtlPolicyEstimateRule for RollingTtlPolicyEstimateNamespaceLengthRule {
    fn check(&self, item: &RollingTtlPolicyEstimate) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyEstimateRequiredLabelRule {
    pub message: String,
}

impl RollingTtlPolicyEstimateRule for RollingTtlPolicyEstimateRequiredLabelRule {
    fn check(&self, item: &RollingTtlPolicyEstimate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyEstimateRequiredNamespaceRule {
    pub message: String,
}

impl RollingTtlPolicyEstimateRule for RollingTtlPolicyEstimateRequiredNamespaceRule {
    fn check(&self, item: &RollingTtlPolicyEstimate) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyEstimateLabelLengthRule {
    pub max_length: i64,
}

impl RollingTtlPolicyEstimateRule for RollingTtlPolicyEstimateLabelLengthRule {
    fn check(&self, item: &RollingTtlPolicyEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyEstimateActiveGuardRule {
    pub limit: i64,
}

impl RollingTtlPolicyEstimateRule for RollingTtlPolicyEstimateActiveGuardRule {
    fn check(&self, item: &RollingTtlPolicyEstimate) -> Vec<String> {
        if item.active && (item.priority > self.limit) {
            return vec![format!(
                "priority {} exceeds {} while active",
                item.priority, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyEstimateAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl RollingTtlPolicyEstimateRule for RollingTtlPolicyEstimateAllowedCipherRule {
    fn check(&self, item: &RollingTtlPolicyEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is expired", item.cipher)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling ttl policy estimate.
#[derive(Debug, Clone)]
pub struct RollingTtlPolicyEstimateValidator {
    pub rules: Vec<Rc<dyn RollingTtlPolicyEstimateRule>>,
}

impl RollingTtlPolicyEstimateValidator {
    pub fn validate(&self, item: &RollingTtlPolicyEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingTtlPolicyEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingTtlPolicyEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_ttl_policy_estimate_validator() -> RollingTtlPolicyEstimateValidator {
    RollingTtlPolicyEstimateValidator {
        rules: vec![
            Rc::new(RollingTtlPolicyEstimateNamespaceLengthRule { max_length: 64 })
                as Rc<dyn RollingTtlPolicyEstimateRule>,
            Rc::new(RollingTtlPolicyEstimateRequiredLabelRule {
                message: "rolling ttl policy estimate needs this field".to_string(),
            }),
            Rc::new(RollingTtlPolicyEstimateRequiredNamespaceRule {
                message: "rolling ttl policy estimate needs this field".to_string(),
            }),
            Rc::new(RollingTtlPolicyEstimateLabelLengthRule { max_length: 120 }),
            Rc::new(RollingTtlPolicyEstimateActiveGuardRule { limit: 120 }),
            Rc::new(RollingTtlPolicyEstimateAllowedCipherRule { allowed: Vec::new() }),
        ],
    }
}
