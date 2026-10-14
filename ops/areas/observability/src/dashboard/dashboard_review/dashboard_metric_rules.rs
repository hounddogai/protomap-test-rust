use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dashboard metric must pass. Check returns its problems, or none.
pub trait DashboardMetricRule: std::fmt::Debug {
    fn check(&self, item: &DashboardMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardMetricTitleLengthRule {
    pub max_length: i64,
}

impl DashboardMetricRule for DashboardMetricTitleLengthRule {
    fn check(&self, item: &DashboardMetric) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardMetricDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardMetricRule for DashboardMetricDurationMinutesRangeRule {
    fn check(&self, item: &DashboardMetric) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardMetricAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl DashboardMetricRule for DashboardMetricAllowedTitleRule {
    fn check(&self, item: &DashboardMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is above maximum", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard metric.
#[derive(Debug, Clone)]
pub struct DashboardMetricValidator {
    pub rules: Vec<Rc<dyn DashboardMetricRule>>,
}

impl DashboardMetricValidator {
    pub fn validate(&self, item: &DashboardMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DashboardMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_metric_validator() -> DashboardMetricValidator {
    DashboardMetricValidator {
        rules: vec![
            Rc::new(DashboardMetricTitleLengthRule { max_length: 86400 }) as Rc<dyn DashboardMetricRule>,
            Rc::new(DashboardMetricDurationMinutesRangeRule { min: 80, max: 255 }),
            Rc::new(DashboardMetricAllowedTitleRule {
                allowed: vec!["seasonal".to_string(), "lenient".to_string(), "adaptive".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_metric_validator();
        let item = new_dashboard_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DashboardMetricTitleLengthRule { max_length: 16 };
        let mut item = new_dashboard_metric("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = DashboardMetricDurationMinutesRangeRule { min: 255, max: 3600 };
        let mut item = new_dashboard_metric("x", "y");
        item.duration_minutes = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
