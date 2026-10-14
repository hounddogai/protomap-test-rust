use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard checkpoint must pass. Check returns its problems, or none.
pub trait DashboardCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &DashboardCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardCheckpointRequiredCodeRule {
    pub message: String,
}

impl DashboardCheckpointRule for DashboardCheckpointRequiredCodeRule {
    fn check(&self, item: &DashboardCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl DashboardCheckpointRule for DashboardCheckpointCodeLengthRule {
    fn check(&self, item: &DashboardCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardCheckpointNameLengthRule {
    pub max_length: i64,
}

impl DashboardCheckpointRule for DashboardCheckpointNameLengthRule {
    fn check(&self, item: &DashboardCheckpoint) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardCheckpointRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardCheckpointRule for DashboardCheckpointRotationDaysRangeRule {
    fn check(&self, item: &DashboardCheckpoint) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardCheckpointRequiredContactEmailRule {
    pub message: String,
}

impl DashboardCheckpointRule for DashboardCheckpointRequiredContactEmailRule {
    fn check(&self, item: &DashboardCheckpoint) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardCheckpointCodePrefixRule {
    pub prefix: String,
}

impl DashboardCheckpointRule for DashboardCheckpointCodePrefixRule {
    fn check(&self, item: &DashboardCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard checkpoint.
#[derive(Debug, Clone)]
pub struct DashboardCheckpointValidator {
    pub rules: Vec<Rc<dyn DashboardCheckpointRule>>,
}

impl DashboardCheckpointValidator {
    pub fn validate(&self, item: &DashboardCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DashboardCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_checkpoint_validator() -> DashboardCheckpointValidator {
    DashboardCheckpointValidator {
        rules: vec![
            Rc::new(DashboardCheckpointRequiredCodeRule {
                message: "dashboard checkpoint needs this field".to_string(),
            }) as Rc<dyn DashboardCheckpointRule>,
            Rc::new(DashboardCheckpointCodeLengthRule { max_length: 3600 }),
            Rc::new(DashboardCheckpointNameLengthRule { max_length: 64 }),
            Rc::new(DashboardCheckpointRotationDaysRangeRule { min: 32, max: 120 }),
            Rc::new(DashboardCheckpointRequiredContactEmailRule {
                message: "dashboard checkpoint needs this field".to_string(),
            }),
            Rc::new(DashboardCheckpointCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
