use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default migration summary must pass. Check returns its problems, or none.
pub trait DefaultMigrationSummaryRule: std::fmt::Debug {
    fn check(&self, item: &DefaultMigrationSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationSummaryRequiredNameRule {
    pub message: String,
}

impl DefaultMigrationSummaryRule for DefaultMigrationSummaryRequiredNameRule {
    fn check(&self, item: &DefaultMigrationSummary) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationSummaryUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultMigrationSummaryRule for DefaultMigrationSummaryUpdatedAtRangeRule {
    fn check(&self, item: &DefaultMigrationSummary) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationSummaryCodePrefixRule {
    pub prefix: String,
}

impl DefaultMigrationSummaryRule for DefaultMigrationSummaryCodePrefixRule {
    fn check(&self, item: &DefaultMigrationSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default migration summary.
#[derive(Debug, Clone)]
pub struct DefaultMigrationSummaryValidator {
    pub rules: Vec<Rc<dyn DefaultMigrationSummaryRule>>,
}

impl DefaultMigrationSummaryValidator {
    pub fn validate(&self, item: &DefaultMigrationSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultMigrationSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultMigrationSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_migration_summary_validator() -> DefaultMigrationSummaryValidator {
    DefaultMigrationSummaryValidator {
        rules: vec![
            Rc::new(DefaultMigrationSummaryRequiredNameRule {
                message: "default migration summary needs an entry".to_string(),
            }) as Rc<dyn DefaultMigrationSummaryRule>,
            Rc::new(DefaultMigrationSummaryUpdatedAtRangeRule { min: 16, max: 3600 }),
            Rc::new(DefaultMigrationSummaryCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
