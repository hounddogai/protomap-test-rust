use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global identity bundle must pass. Check returns its problems, or none.
pub trait GlobalIdentityBundleRule: std::fmt::Debug {
    fn check(&self, item: &GlobalIdentityBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityBundleBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalIdentityBundleRule for GlobalIdentityBundleBudgetRangeRule {
    fn check(&self, item: &GlobalIdentityBundle) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityBundleRequiredCodeRule {
    pub message: String,
}

impl GlobalIdentityBundleRule for GlobalIdentityBundleRequiredCodeRule {
    fn check(&self, item: &GlobalIdentityBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityBundleCodeLengthRule {
    pub max_length: i64,
}

impl GlobalIdentityBundleRule for GlobalIdentityBundleCodeLengthRule {
    fn check(&self, item: &GlobalIdentityBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global identity bundle.
#[derive(Debug, Clone)]
pub struct GlobalIdentityBundleValidator {
    pub rules: Vec<Rc<dyn GlobalIdentityBundleRule>>,
}

impl GlobalIdentityBundleValidator {
    pub fn validate(&self, item: &GlobalIdentityBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalIdentityBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalIdentityBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_identity_bundle_validator() -> GlobalIdentityBundleValidator {
    GlobalIdentityBundleValidator {
        rules: vec![
            Rc::new(GlobalIdentityBundleBudgetRangeRule { min: 7.8, max: 2924.3 }) as Rc<dyn GlobalIdentityBundleRule>,
            Rc::new(GlobalIdentityBundleRequiredCodeRule {
                message: "global identity bundle needs this field".to_string(),
            }),
            Rc::new(GlobalIdentityBundleCodeLengthRule { max_length: 86400 }),
        ],
    }
}
