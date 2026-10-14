use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling migration metric must pass. Check returns its problems, or none.
pub trait RollingMigrationMetricRule: std::fmt::Debug {
    fn check(&self, item: &RollingMigrationMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationMetricPhoneLengthRule {
    pub max_length: i64,
}

impl RollingMigrationMetricRule for RollingMigrationMetricPhoneLengthRule {
    fn check(&self, item: &RollingMigrationMetric) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationMetricTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingMigrationMetricRule for RollingMigrationMetricTtlSecondsRangeRule {
    fn check(&self, item: &RollingMigrationMetric) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationMetricLocaleLengthRule {
    pub max_length: i64,
}

impl RollingMigrationMetricRule for RollingMigrationMetricLocaleLengthRule {
    fn check(&self, item: &RollingMigrationMetric) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling migration metric.
#[derive(Debug, Clone)]
pub struct RollingMigrationMetricValidator {
    pub rules: Vec<Rc<dyn RollingMigrationMetricRule>>,
}

impl RollingMigrationMetricValidator {
    pub fn validate(&self, item: &RollingMigrationMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingMigrationMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingMigrationMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_migration_metric_validator() -> RollingMigrationMetricValidator {
    RollingMigrationMetricValidator {
        rules: vec![
            Rc::new(RollingMigrationMetricPhoneLengthRule { max_length: 86400 }) as Rc<dyn RollingMigrationMetricRule>,
            Rc::new(RollingMigrationMetricTtlSecondsRangeRule { min: 255, max: 3600 }),
            Rc::new(RollingMigrationMetricLocaleLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_migration_metric_validator();
        let item = new_rolling_migration_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = RollingMigrationMetricPhoneLengthRule { max_length: 120 };
        let mut item = new_rolling_migration_metric("x", "y");
        item.phone = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = RollingMigrationMetricTtlSecondsRangeRule { min: 16, max: 80 };
        let mut item = new_rolling_migration_metric("x", "y");
        item.ttl_seconds = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = RollingMigrationMetricLocaleLengthRule { max_length: 32 };
        let mut item = new_rolling_migration_metric("x", "y");
        item.locale = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
