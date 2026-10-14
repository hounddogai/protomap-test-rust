use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a binding policy must pass. Check returns its problems, or none.
pub trait BindingPolicyRule: std::fmt::Debug {
    fn check(&self, item: &BindingPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingPolicyAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl BindingPolicyRule for BindingPolicyAllowedReviewerRule {
    fn check(&self, item: &BindingPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is expired", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingPolicyCategoryLengthRule {
    pub max_length: i64,
}

impl BindingPolicyRule for BindingPolicyCategoryLengthRule {
    fn check(&self, item: &BindingPolicy) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingPolicyRequiredCodeRule {
    pub message: String,
}

impl BindingPolicyRule for BindingPolicyRequiredCodeRule {
    fn check(&self, item: &BindingPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingPolicyPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingPolicyRule for BindingPolicyPriorityRangeRule {
    fn check(&self, item: &BindingPolicy) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingPolicyCodeLengthRule {
    pub max_length: i64,
}

impl BindingPolicyRule for BindingPolicyCodeLengthRule {
    fn check(&self, item: &BindingPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingPolicyWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingPolicyRule for BindingPolicyWindowDaysRangeRule {
    fn check(&self, item: &BindingPolicy) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a binding policy.
#[derive(Debug, Clone)]
pub struct BindingPolicyValidator {
    pub rules: Vec<Rc<dyn BindingPolicyRule>>,
}

impl BindingPolicyValidator {
    pub fn validate(&self, item: &BindingPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_policy_validator() -> BindingPolicyValidator {
    BindingPolicyValidator {
        rules: vec![
            Rc::new(BindingPolicyAllowedReviewerRule {
                allowed: vec!["legacy".to_string()],
            }) as Rc<dyn BindingPolicyRule>,
            Rc::new(BindingPolicyCategoryLengthRule { max_length: 64 }),
            Rc::new(BindingPolicyRequiredCodeRule {
                message: "binding policy needs this field".to_string(),
            }),
            Rc::new(BindingPolicyPriorityRangeRule { min: 64, max: 64 }),
            Rc::new(BindingPolicyCodeLengthRule { max_length: 3600 }),
            Rc::new(BindingPolicyWindowDaysRangeRule { min: 32, max: 80 }),
        ],
    }
}
