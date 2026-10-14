use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding profile must pass. Check returns its problems, or none.
pub trait BindingProfileRule: std::fmt::Debug {
    fn check(&self, item: &BindingProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingProfileRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingProfileRule for BindingProfileRetriesRangeRule {
    fn check(&self, item: &BindingProfile) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingProfileRequiredLabelRule {
    pub message: String,
}

impl BindingProfileRule for BindingProfileRequiredLabelRule {
    fn check(&self, item: &BindingProfile) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingProfileActiveGuardRule {
    pub limit: i64,
}

impl BindingProfileRule for BindingProfileActiveGuardRule {
    fn check(&self, item: &BindingProfile) -> Vec<String> {
        if item.active && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while active",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingProfileRequiredCodeRule {
    pub message: String,
}

impl BindingProfileRule for BindingProfileRequiredCodeRule {
    fn check(&self, item: &BindingProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding profile.
#[derive(Debug, Clone)]
pub struct BindingProfileValidator {
    pub rules: Vec<Rc<dyn BindingProfileRule>>,
}

impl BindingProfileValidator {
    pub fn validate(&self, item: &BindingProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_profile_validator() -> BindingProfileValidator {
    BindingProfileValidator {
        rules: vec![
            Rc::new(BindingProfileRetriesRangeRule { min: 16, max: 80 }) as Rc<dyn BindingProfileRule>,
            Rc::new(BindingProfileRequiredLabelRule {
                message: "binding profile needs a value".to_string(),
            }),
            Rc::new(BindingProfileActiveGuardRule { limit: 86400 }),
            Rc::new(BindingProfileRequiredCodeRule {
                message: "binding profile needs this field".to_string(),
            }),
        ],
    }
}
