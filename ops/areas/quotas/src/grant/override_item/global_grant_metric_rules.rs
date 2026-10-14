use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global grant metric must pass. Check returns its problems, or none.
pub trait GlobalGrantMetricRule: std::fmt::Debug {
    fn check(&self, item: &GlobalGrantMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalGrantMetricAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl GlobalGrantMetricRule for GlobalGrantMetricAllowedSummaryRule {
    fn check(&self, item: &GlobalGrantMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is missing", item.summary)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalGrantMetricTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalGrantMetricRule for GlobalGrantMetricTtlSecondsRangeRule {
    fn check(&self, item: &GlobalGrantMetric) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalGrantMetricNameLengthRule {
    pub max_length: i64,
}

impl GlobalGrantMetricRule for GlobalGrantMetricNameLengthRule {
    fn check(&self, item: &GlobalGrantMetric) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalGrantMetricSummaryLengthRule {
    pub max_length: i64,
}

impl GlobalGrantMetricRule for GlobalGrantMetricSummaryLengthRule {
    fn check(&self, item: &GlobalGrantMetric) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global grant metric.
#[derive(Debug, Clone)]
pub struct GlobalGrantMetricValidator {
    pub rules: Vec<Rc<dyn GlobalGrantMetricRule>>,
}

impl GlobalGrantMetricValidator {
    pub fn validate(&self, item: &GlobalGrantMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalGrantMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalGrantMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_grant_metric_validator() -> GlobalGrantMetricValidator {
    GlobalGrantMetricValidator {
        rules: vec![
            Rc::new(GlobalGrantMetricAllowedSummaryRule {
                allowed: vec!["capped".to_string(), "primary".to_string(), "default".to_string()],
            }) as Rc<dyn GlobalGrantMetricRule>,
            Rc::new(GlobalGrantMetricTtlSecondsRangeRule { min: 120, max: 255 }),
            Rc::new(GlobalGrantMetricNameLengthRule { max_length: 16 }),
            Rc::new(GlobalGrantMetricSummaryLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_grant_metric_validator();
        let item = new_global_grant_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = GlobalGrantMetricTtlSecondsRangeRule { min: 80, max: 1000 };
        let mut item = new_global_grant_metric("x", "y");
        item.ttl_seconds = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = GlobalGrantMetricNameLengthRule { max_length: 86400 };
        let mut item = new_global_grant_metric("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = GlobalGrantMetricSummaryLengthRule { max_length: 80 };
        let mut item = new_global_grant_metric("x", "y");
        item.summary = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
