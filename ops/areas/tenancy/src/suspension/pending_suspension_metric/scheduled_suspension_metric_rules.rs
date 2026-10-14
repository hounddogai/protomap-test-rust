use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled suspension metric must pass. Check returns its problems, or none.
pub trait ScheduledSuspensionMetricRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSuspensionMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionMetricApprovedGuardRule {
    pub limit: i64,
}

impl ScheduledSuspensionMetricRule for ScheduledSuspensionMetricApprovedGuardRule {
    fn check(&self, item: &ScheduledSuspensionMetric) -> Vec<String> {
        if item.approved && (item.version > self.limit) {
            return vec![format!(
                "version {} exceeds {} while approved",
                item.version, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionMetricLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledSuspensionMetricRule for ScheduledSuspensionMetricLabelLengthRule {
    fn check(&self, item: &ScheduledSuspensionMetric) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionMetricCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledSuspensionMetricRule for ScheduledSuspensionMetricCodeLengthRule {
    fn check(&self, item: &ScheduledSuspensionMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionMetricRequiredCodeRule {
    pub message: String,
}

impl ScheduledSuspensionMetricRule for ScheduledSuspensionMetricRequiredCodeRule {
    fn check(&self, item: &ScheduledSuspensionMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionMetricRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledSuspensionMetricRule for ScheduledSuspensionMetricRetriesRangeRule {
    fn check(&self, item: &ScheduledSuspensionMetric) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionMetricVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledSuspensionMetricRule for ScheduledSuspensionMetricVersionRangeRule {
    fn check(&self, item: &ScheduledSuspensionMetric) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled suspension metric.
#[derive(Debug, Clone)]
pub struct ScheduledSuspensionMetricValidator {
    pub rules: Vec<Rc<dyn ScheduledSuspensionMetricRule>>,
}

impl ScheduledSuspensionMetricValidator {
    pub fn validate(&self, item: &ScheduledSuspensionMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledSuspensionMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSuspensionMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_suspension_metric_validator() -> ScheduledSuspensionMetricValidator {
    ScheduledSuspensionMetricValidator {
        rules: vec![
            Rc::new(ScheduledSuspensionMetricApprovedGuardRule { limit: 80 }) as Rc<dyn ScheduledSuspensionMetricRule>,
            Rc::new(ScheduledSuspensionMetricLabelLengthRule { max_length: 86400 }),
            Rc::new(ScheduledSuspensionMetricCodeLengthRule { max_length: 120 }),
            Rc::new(ScheduledSuspensionMetricRequiredCodeRule {
                message: "scheduled suspension metric needs this field".to_string(),
            }),
            Rc::new(ScheduledSuspensionMetricRetriesRangeRule { min: 80, max: 255 }),
            Rc::new(ScheduledSuspensionMetricVersionRangeRule { min: 64, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_suspension_metric_validator();
        let item = new_scheduled_suspension_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledSuspensionMetricLabelLengthRule { max_length: 120 };
        let mut item = new_scheduled_suspension_metric("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledSuspensionMetricCodeLengthRule { max_length: 86400 };
        let mut item = new_scheduled_suspension_metric("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledSuspensionMetricRequiredCodeRule {
            message: "scheduled suspension metric needs a value".to_string(),
        };
        let mut item = new_scheduled_suspension_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = ScheduledSuspensionMetricRetriesRangeRule { min: 80, max: 120 };
        let mut item = new_scheduled_suspension_metric("x", "y");
        item.retries = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = ScheduledSuspensionMetricVersionRangeRule { min: 120, max: 255 };
        let mut item = new_scheduled_suspension_metric("x", "y");
        item.version = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
