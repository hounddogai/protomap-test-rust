use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default migration variance must pass. Check returns its problems, or none.
pub trait DefaultMigrationVarianceRule: std::fmt::Debug {
    fn check(&self, item: &DefaultMigrationVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationVarianceCodeLengthRule {
    pub max_length: i64,
}

impl DefaultMigrationVarianceRule for DefaultMigrationVarianceCodeLengthRule {
    fn check(&self, item: &DefaultMigrationVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationVarianceRequiredTitleRule {
    pub message: String,
}

impl DefaultMigrationVarianceRule for DefaultMigrationVarianceRequiredTitleRule {
    fn check(&self, item: &DefaultMigrationVariance) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationVarianceAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl DefaultMigrationVarianceRule for DefaultMigrationVarianceAllowedSegmentRule {
    fn check(&self, item: &DefaultMigrationVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is below minimum", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationVarianceTitleLengthRule {
    pub max_length: i64,
}

impl DefaultMigrationVarianceRule for DefaultMigrationVarianceTitleLengthRule {
    fn check(&self, item: &DefaultMigrationVariance) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationVarianceFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultMigrationVarianceRule for DefaultMigrationVarianceFloorValueRangeRule {
    fn check(&self, item: &DefaultMigrationVariance) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a default migration variance.
#[derive(Debug, Clone)]
pub struct DefaultMigrationVarianceValidator {
    pub rules: Vec<Rc<dyn DefaultMigrationVarianceRule>>,
}

impl DefaultMigrationVarianceValidator {
    pub fn validate(&self, item: &DefaultMigrationVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultMigrationVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultMigrationVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_migration_variance_validator() -> DefaultMigrationVarianceValidator {
    DefaultMigrationVarianceValidator {
        rules: vec![
            Rc::new(DefaultMigrationVarianceCodeLengthRule { max_length: 64 }) as Rc<dyn DefaultMigrationVarianceRule>,
            Rc::new(DefaultMigrationVarianceRequiredTitleRule {
                message: "default migration variance needs a value".to_string(),
            }),
            Rc::new(DefaultMigrationVarianceAllowedSegmentRule {
                allowed: vec!["capped".to_string(), "manual".to_string()],
            }),
            Rc::new(DefaultMigrationVarianceTitleLengthRule { max_length: 86400 }),
            Rc::new(DefaultMigrationVarianceFloorValueRangeRule { min: 3.8, max: 2479.3 }),
        ],
    }
}
