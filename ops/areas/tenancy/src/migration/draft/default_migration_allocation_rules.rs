use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default migration allocation must pass. Check returns its problems, or none.
pub trait DefaultMigrationAllocationRule: std::fmt::Debug {
    fn check(&self, item: &DefaultMigrationAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationAllocationVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultMigrationAllocationRule for DefaultMigrationAllocationVersionRangeRule {
    fn check(&self, item: &DefaultMigrationAllocation) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationAllocationCodeLengthRule {
    pub max_length: i64,
}

impl DefaultMigrationAllocationRule for DefaultMigrationAllocationCodeLengthRule {
    fn check(&self, item: &DefaultMigrationAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationAllocationRequiredCodeRule {
    pub message: String,
}

impl DefaultMigrationAllocationRule for DefaultMigrationAllocationRequiredCodeRule {
    fn check(&self, item: &DefaultMigrationAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default migration allocation.
#[derive(Debug, Clone)]
pub struct DefaultMigrationAllocationValidator {
    pub rules: Vec<Rc<dyn DefaultMigrationAllocationRule>>,
}

impl DefaultMigrationAllocationValidator {
    pub fn validate(&self, item: &DefaultMigrationAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultMigrationAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultMigrationAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_migration_allocation_validator() -> DefaultMigrationAllocationValidator {
    DefaultMigrationAllocationValidator {
        rules: vec![
            Rc::new(DefaultMigrationAllocationVersionRangeRule { min: 16, max: 120 })
                as Rc<dyn DefaultMigrationAllocationRule>,
            Rc::new(DefaultMigrationAllocationCodeLengthRule { max_length: 80 }),
            Rc::new(DefaultMigrationAllocationRequiredCodeRule {
                message: "default migration allocation needs this field".to_string(),
            }),
        ],
    }
}
