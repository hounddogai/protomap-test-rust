use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy usage group must pass. Check returns its problems, or none.
pub trait LegacyUsageGroupRule: std::fmt::Debug {
    fn check(&self, item: &LegacyUsageGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageGroupRequiredUnitRule {
    pub message: String,
}

impl LegacyUsageGroupRule for LegacyUsageGroupRequiredUnitRule {
    fn check(&self, item: &LegacyUsageGroup) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageGroupRequiredCodeRule {
    pub message: String,
}

impl LegacyUsageGroupRule for LegacyUsageGroupRequiredCodeRule {
    fn check(&self, item: &LegacyUsageGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageGroupRequiredNameRule {
    pub message: String,
}

impl LegacyUsageGroupRule for LegacyUsageGroupRequiredNameRule {
    fn check(&self, item: &LegacyUsageGroup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageGroupUnitLengthRule {
    pub max_length: i64,
}

impl LegacyUsageGroupRule for LegacyUsageGroupUnitLengthRule {
    fn check(&self, item: &LegacyUsageGroup) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy usage group.
#[derive(Debug, Clone)]
pub struct LegacyUsageGroupValidator {
    pub rules: Vec<Rc<dyn LegacyUsageGroupRule>>,
}

impl LegacyUsageGroupValidator {
    pub fn validate(&self, item: &LegacyUsageGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyUsageGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyUsageGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_usage_group_validator() -> LegacyUsageGroupValidator {
    LegacyUsageGroupValidator {
        rules: vec![
            Rc::new(LegacyUsageGroupRequiredUnitRule {
                message: "legacy usage group needs a value".to_string(),
            }) as Rc<dyn LegacyUsageGroupRule>,
            Rc::new(LegacyUsageGroupRequiredCodeRule {
                message: "legacy usage group needs a value".to_string(),
            }),
            Rc::new(LegacyUsageGroupRequiredNameRule {
                message: "legacy usage group needs a value".to_string(),
            }),
            Rc::new(LegacyUsageGroupUnitLengthRule { max_length: 86400 }),
        ],
    }
}
