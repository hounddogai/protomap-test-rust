use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled egress policy must pass. Check returns its problems, or none.
pub trait ScheduledEgressPolicyRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledEgressPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressPolicyReviewerLengthRule {
    pub max_length: i64,
}

impl ScheduledEgressPolicyRule for ScheduledEgressPolicyReviewerLengthRule {
    fn check(&self, item: &ScheduledEgressPolicy) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressPolicyAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledEgressPolicyRule for ScheduledEgressPolicyAmountRangeRule {
    fn check(&self, item: &ScheduledEgressPolicy) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressPolicyRequiredReviewerRule {
    pub message: String,
}

impl ScheduledEgressPolicyRule for ScheduledEgressPolicyRequiredReviewerRule {
    fn check(&self, item: &ScheduledEgressPolicy) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressPolicyRequiredClusterRule {
    pub message: String,
}

impl ScheduledEgressPolicyRule for ScheduledEgressPolicyRequiredClusterRule {
    fn check(&self, item: &ScheduledEgressPolicy) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressPolicyCodePrefixRule {
    pub prefix: String,
}

impl ScheduledEgressPolicyRule for ScheduledEgressPolicyCodePrefixRule {
    fn check(&self, item: &ScheduledEgressPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled egress policy.
#[derive(Debug, Clone)]
pub struct ScheduledEgressPolicyValidator {
    pub rules: Vec<Rc<dyn ScheduledEgressPolicyRule>>,
}

impl ScheduledEgressPolicyValidator {
    pub fn validate(&self, item: &ScheduledEgressPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledEgressPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledEgressPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_egress_policy_validator() -> ScheduledEgressPolicyValidator {
    ScheduledEgressPolicyValidator {
        rules: vec![
            Rc::new(ScheduledEgressPolicyReviewerLengthRule { max_length: 80 }) as Rc<dyn ScheduledEgressPolicyRule>,
            Rc::new(ScheduledEgressPolicyAmountRangeRule { min: 7.3, max: 789.0 }),
            Rc::new(ScheduledEgressPolicyRequiredReviewerRule {
                message: "scheduled egress policy needs this field".to_string(),
            }),
            Rc::new(ScheduledEgressPolicyRequiredClusterRule {
                message: "scheduled egress policy needs this field".to_string(),
            }),
            Rc::new(ScheduledEgressPolicyCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
