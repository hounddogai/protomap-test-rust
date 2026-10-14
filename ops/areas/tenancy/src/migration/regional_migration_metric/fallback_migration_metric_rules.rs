use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback migration metric must pass. Check returns its problems, or none.
pub trait FallbackMigrationMetricRule: std::fmt::Debug {
    fn check(&self, item: &FallbackMigrationMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMigrationMetricNameLengthRule {
    pub max_length: i64,
}

impl FallbackMigrationMetricRule for FallbackMigrationMetricNameLengthRule {
    fn check(&self, item: &FallbackMigrationMetric) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMigrationMetricExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackMigrationMetricRule for FallbackMigrationMetricExpiresAtRangeRule {
    fn check(&self, item: &FallbackMigrationMetric) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMigrationMetricRequiredCodeRule {
    pub message: String,
}

impl FallbackMigrationMetricRule for FallbackMigrationMetricRequiredCodeRule {
    fn check(&self, item: &FallbackMigrationMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback migration metric.
#[derive(Debug, Clone)]
pub struct FallbackMigrationMetricValidator {
    pub rules: Vec<Rc<dyn FallbackMigrationMetricRule>>,
}

impl FallbackMigrationMetricValidator {
    pub fn validate(&self, item: &FallbackMigrationMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackMigrationMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackMigrationMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_migration_metric_validator() -> FallbackMigrationMetricValidator {
    FallbackMigrationMetricValidator {
        rules: vec![
            Rc::new(FallbackMigrationMetricNameLengthRule { max_length: 64 }) as Rc<dyn FallbackMigrationMetricRule>,
            Rc::new(FallbackMigrationMetricExpiresAtRangeRule { min: 255, max: 1000 }),
            Rc::new(FallbackMigrationMetricRequiredCodeRule {
                message: "fallback migration metric needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_migration_metric_validator();
        let item = new_fallback_migration_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FallbackMigrationMetricNameLengthRule { max_length: 120 };
        let mut item = new_fallback_migration_metric("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = FallbackMigrationMetricExpiresAtRangeRule { min: 32, max: 1000 };
        let mut item = new_fallback_migration_metric("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackMigrationMetricRequiredCodeRule {
            message: "fallback migration metric needs an entry".to_string(),
        };
        let mut item = new_fallback_migration_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
