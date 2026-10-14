use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default migration metric must pass. Check returns its problems, or none.
pub trait DefaultMigrationMetricRule: std::fmt::Debug {
    fn check(&self, item: &DefaultMigrationMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationMetricRequiredLabelRule {
    pub message: String,
}

impl DefaultMigrationMetricRule for DefaultMigrationMetricRequiredLabelRule {
    fn check(&self, item: &DefaultMigrationMetric) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationMetricRequiredTenantIdRule {
    pub message: String,
}

impl DefaultMigrationMetricRule for DefaultMigrationMetricRequiredTenantIdRule {
    fn check(&self, item: &DefaultMigrationMetric) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationMetricAudienceLengthRule {
    pub max_length: i64,
}

impl DefaultMigrationMetricRule for DefaultMigrationMetricAudienceLengthRule {
    fn check(&self, item: &DefaultMigrationMetric) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a default migration metric.
#[derive(Debug, Clone)]
pub struct DefaultMigrationMetricValidator {
    pub rules: Vec<Rc<dyn DefaultMigrationMetricRule>>,
}

impl DefaultMigrationMetricValidator {
    pub fn validate(&self, item: &DefaultMigrationMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultMigrationMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultMigrationMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_migration_metric_validator() -> DefaultMigrationMetricValidator {
    DefaultMigrationMetricValidator {
        rules: vec![
            Rc::new(DefaultMigrationMetricRequiredLabelRule {
                message: "default migration metric needs an entry".to_string(),
            }) as Rc<dyn DefaultMigrationMetricRule>,
            Rc::new(DefaultMigrationMetricRequiredTenantIdRule {
                message: "default migration metric needs an entry".to_string(),
            }),
            Rc::new(DefaultMigrationMetricAudienceLengthRule { max_length: 1000 }),
        ],
    }
}
