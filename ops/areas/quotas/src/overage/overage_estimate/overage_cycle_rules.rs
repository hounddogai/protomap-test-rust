use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage cycle must pass. Check returns its problems, or none.
pub trait OverageCycleRule: std::fmt::Debug {
    fn check(&self, item: &OverageCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageCycleRequiredCategoryRule {
    pub message: String,
}

impl OverageCycleRule for OverageCycleRequiredCategoryRule {
    fn check(&self, item: &OverageCycle) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageCycleRequiredChannelRule {
    pub message: String,
}

impl OverageCycleRule for OverageCycleRequiredChannelRule {
    fn check(&self, item: &OverageCycle) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageCyclePinnedGuardRule {
    pub limit: i64,
}

impl OverageCycleRule for OverageCyclePinnedGuardRule {
    fn check(&self, item: &OverageCycle) -> Vec<String> {
        if item.pinned && (item.capacity > self.limit) {
            return vec![format!(
                "capacity {} exceeds {} while pinned",
                item.capacity, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage cycle.
#[derive(Debug, Clone)]
pub struct OverageCycleValidator {
    pub rules: Vec<Rc<dyn OverageCycleRule>>,
}

impl OverageCycleValidator {
    pub fn validate(&self, item: &OverageCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_cycle_validator() -> OverageCycleValidator {
    OverageCycleValidator {
        rules: vec![
            Rc::new(OverageCycleRequiredCategoryRule {
                message: "overage cycle needs a value".to_string(),
            }) as Rc<dyn OverageCycleRule>,
            Rc::new(OverageCycleRequiredChannelRule {
                message: "overage cycle needs a value".to_string(),
            }),
            Rc::new(OverageCyclePinnedGuardRule { limit: 80 }),
        ],
    }
}
