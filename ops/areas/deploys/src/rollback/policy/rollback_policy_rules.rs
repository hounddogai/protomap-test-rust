use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollback policy must pass. Check returns its problems, or none.
pub trait RollbackPolicyRule: std::fmt::Debug {
    fn check(&self, item: &RollbackPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPolicySequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackPolicyRule for RollbackPolicySequenceRangeRule {
    fn check(&self, item: &RollbackPolicy) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPolicyRequiredCodeRule {
    pub message: String,
}

impl RollbackPolicyRule for RollbackPolicyRequiredCodeRule {
    fn check(&self, item: &RollbackPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPolicyCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollbackPolicyRule for RollbackPolicyCostRangeRule {
    fn check(&self, item: &RollbackPolicy) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPolicyAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl RollbackPolicyRule for RollbackPolicyAllowedKeyIdRule {
    fn check(&self, item: &RollbackPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is missing", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPolicyKeyIdLengthRule {
    pub max_length: i64,
}

impl RollbackPolicyRule for RollbackPolicyKeyIdLengthRule {
    fn check(&self, item: &RollbackPolicy) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback policy.
#[derive(Debug, Clone)]
pub struct RollbackPolicyValidator {
    pub rules: Vec<Rc<dyn RollbackPolicyRule>>,
}

impl RollbackPolicyValidator {
    pub fn validate(&self, item: &RollbackPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_policy_validator() -> RollbackPolicyValidator {
    RollbackPolicyValidator {
        rules: vec![
            Rc::new(RollbackPolicySequenceRangeRule { min: 80, max: 86400 }) as Rc<dyn RollbackPolicyRule>,
            Rc::new(RollbackPolicyRequiredCodeRule {
                message: "rollback policy needs a value".to_string(),
            }),
            Rc::new(RollbackPolicyCostRangeRule { min: 0.1, max: 1371.8 }),
            Rc::new(RollbackPolicyAllowedKeyIdRule {
                allowed: vec!["adaptive".to_string(), "scheduled".to_string(), "lenient".to_string()],
            }),
            Rc::new(RollbackPolicyKeyIdLengthRule { max_length: 64 }),
        ],
    }
}
