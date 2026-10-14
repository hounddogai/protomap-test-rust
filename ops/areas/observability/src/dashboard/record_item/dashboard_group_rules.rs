use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard group must pass. Check returns its problems, or none.
pub trait DashboardGroupRule: std::fmt::Debug {
    fn check(&self, item: &DashboardGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardGroupRequiredCodeRule {
    pub message: String,
}

impl DashboardGroupRule for DashboardGroupRequiredCodeRule {
    fn check(&self, item: &DashboardGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardGroupCodeLengthRule {
    pub max_length: i64,
}

impl DashboardGroupRule for DashboardGroupCodeLengthRule {
    fn check(&self, item: &DashboardGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardGroupRequiredTitleRule {
    pub message: String,
}

impl DashboardGroupRule for DashboardGroupRequiredTitleRule {
    fn check(&self, item: &DashboardGroup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardGroupActiveGuardRule {
    pub limit: i64,
}

impl DashboardGroupRule for DashboardGroupActiveGuardRule {
    fn check(&self, item: &DashboardGroup) -> Vec<String> {
        if item.active && (item.burst > self.limit) {
            return vec![format!("burst {} exceeds {} while active", item.burst, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard group.
#[derive(Debug, Clone)]
pub struct DashboardGroupValidator {
    pub rules: Vec<Rc<dyn DashboardGroupRule>>,
}

impl DashboardGroupValidator {
    pub fn validate(&self, item: &DashboardGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DashboardGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_group_validator() -> DashboardGroupValidator {
    DashboardGroupValidator {
        rules: vec![
            Rc::new(DashboardGroupRequiredCodeRule {
                message: "dashboard group needs this field".to_string(),
            }) as Rc<dyn DashboardGroupRule>,
            Rc::new(DashboardGroupCodeLengthRule { max_length: 120 }),
            Rc::new(DashboardGroupRequiredTitleRule {
                message: "dashboard group needs this field".to_string(),
            }),
            Rc::new(DashboardGroupActiveGuardRule { limit: 255 }),
        ],
    }
}
