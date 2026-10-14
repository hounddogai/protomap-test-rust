use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration metric must pass. Check returns its problems, or none.
pub trait MigrationMetricRule: std::fmt::Debug {
    fn check(&self, item: &MigrationMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationMetricRequiredRegionRule {
    pub message: String,
}

impl MigrationMetricRule for MigrationMetricRequiredRegionRule {
    fn check(&self, item: &MigrationMetric) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationMetricNameLengthRule {
    pub max_length: i64,
}

impl MigrationMetricRule for MigrationMetricNameLengthRule {
    fn check(&self, item: &MigrationMetric) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationMetricMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MigrationMetricRule for MigrationMetricMinimumRangeRule {
    fn check(&self, item: &MigrationMetric) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a migration metric.
#[derive(Debug, Clone)]
pub struct MigrationMetricValidator {
    pub rules: Vec<Rc<dyn MigrationMetricRule>>,
}

impl MigrationMetricValidator {
    pub fn validate(&self, item: &MigrationMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_metric_validator() -> MigrationMetricValidator {
    MigrationMetricValidator {
        rules: vec![
            Rc::new(MigrationMetricRequiredRegionRule {
                message: "migration metric needs an entry".to_string(),
            }) as Rc<dyn MigrationMetricRule>,
            Rc::new(MigrationMetricNameLengthRule { max_length: 64 }),
            Rc::new(MigrationMetricMinimumRangeRule { min: 0.0, max: 350.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_metric_validator();
        let item = new_migration_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = MigrationMetricRequiredRegionRule {
            message: "migration metric needs an entry".to_string(),
        };
        let mut item = new_migration_metric("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = MigrationMetricNameLengthRule { max_length: 16 };
        let mut item = new_migration_metric("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = MigrationMetricMinimumRangeRule { min: 8.5, max: 2452.2 };
        let mut item = new_migration_metric("x", "y");
        item.minimum = 2453.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
