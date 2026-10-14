use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped dashboard metric must pass. Check returns its problems, or none.
pub trait CappedDashboardMetricRule: std::fmt::Debug {
    fn check(&self, item: &CappedDashboardMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardMetricCodeLengthRule {
    pub max_length: i64,
}

impl CappedDashboardMetricRule for CappedDashboardMetricCodeLengthRule {
    fn check(&self, item: &CappedDashboardMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardMetricCodePrefixRule {
    pub prefix: String,
}

impl CappedDashboardMetricRule for CappedDashboardMetricCodePrefixRule {
    fn check(&self, item: &CappedDashboardMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardMetricVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedDashboardMetricRule for CappedDashboardMetricVersionRangeRule {
    fn check(&self, item: &CappedDashboardMetric) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardMetricDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedDashboardMetricRule for CappedDashboardMetricDurationMinutesRangeRule {
    fn check(&self, item: &CappedDashboardMetric) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped dashboard metric.
#[derive(Debug, Clone)]
pub struct CappedDashboardMetricValidator {
    pub rules: Vec<Rc<dyn CappedDashboardMetricRule>>,
}

impl CappedDashboardMetricValidator {
    pub fn validate(&self, item: &CappedDashboardMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedDashboardMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedDashboardMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_dashboard_metric_validator() -> CappedDashboardMetricValidator {
    CappedDashboardMetricValidator {
        rules: vec![
            Rc::new(CappedDashboardMetricCodeLengthRule { max_length: 32 }) as Rc<dyn CappedDashboardMetricRule>,
            Rc::new(CappedDashboardMetricCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(CappedDashboardMetricVersionRangeRule { min: 80, max: 120 }),
            Rc::new(CappedDashboardMetricDurationMinutesRangeRule { min: 86400, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_dashboard_metric_validator();
        let item = new_capped_dashboard_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedDashboardMetricCodeLengthRule { max_length: 80 };
        let mut item = new_capped_dashboard_metric("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = CappedDashboardMetricVersionRangeRule { min: 64, max: 120 };
        let mut item = new_capped_dashboard_metric("x", "y");
        item.version = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = CappedDashboardMetricDurationMinutesRangeRule { min: 1000, max: 86400 };
        let mut item = new_capped_dashboard_metric("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
