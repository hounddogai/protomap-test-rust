use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy rollback milestone must pass. Check returns its problems, or none.
pub trait LegacyRollbackMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &LegacyRollbackMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackMilestoneCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyRollbackMilestoneRule for LegacyRollbackMilestoneCreatedAtRangeRule {
    fn check(&self, item: &LegacyRollbackMilestone) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackMilestoneRequiredCodeRule {
    pub message: String,
}

impl LegacyRollbackMilestoneRule for LegacyRollbackMilestoneRequiredCodeRule {
    fn check(&self, item: &LegacyRollbackMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackMilestonePhoneLengthRule {
    pub max_length: i64,
}

impl LegacyRollbackMilestoneRule for LegacyRollbackMilestonePhoneLengthRule {
    fn check(&self, item: &LegacyRollbackMilestone) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy rollback milestone.
#[derive(Debug, Clone)]
pub struct LegacyRollbackMilestoneValidator {
    pub rules: Vec<Rc<dyn LegacyRollbackMilestoneRule>>,
}

impl LegacyRollbackMilestoneValidator {
    pub fn validate(&self, item: &LegacyRollbackMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyRollbackMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyRollbackMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_rollback_milestone_validator() -> LegacyRollbackMilestoneValidator {
    LegacyRollbackMilestoneValidator {
        rules: vec![
            Rc::new(LegacyRollbackMilestoneCreatedAtRangeRule { min: 64, max: 80 })
                as Rc<dyn LegacyRollbackMilestoneRule>,
            Rc::new(LegacyRollbackMilestoneRequiredCodeRule {
                message: "legacy rollback milestone needs a value".to_string(),
            }),
            Rc::new(LegacyRollbackMilestonePhoneLengthRule { max_length: 86400 }),
        ],
    }
}
