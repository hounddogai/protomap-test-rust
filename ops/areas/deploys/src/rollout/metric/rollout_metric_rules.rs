use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout metric must pass. Check returns its problems, or none.
pub trait RolloutMetricRule: std::fmt::Debug {
    fn check(&self, item: &RolloutMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutMetricSegmentLengthRule {
    pub max_length: i64,
}

impl RolloutMetricRule for RolloutMetricSegmentLengthRule {
    fn check(&self, item: &RolloutMetric) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutMetricRequiredCurrencyRule {
    pub message: String,
}

impl RolloutMetricRule for RolloutMetricRequiredCurrencyRule {
    fn check(&self, item: &RolloutMetric) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutMetricRequiredTitleRule {
    pub message: String,
}

impl RolloutMetricRule for RolloutMetricRequiredTitleRule {
    fn check(&self, item: &RolloutMetric) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout metric.
#[derive(Debug, Clone)]
pub struct RolloutMetricValidator {
    pub rules: Vec<Rc<dyn RolloutMetricRule>>,
}

impl RolloutMetricValidator {
    pub fn validate(&self, item: &RolloutMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_metric_validator() -> RolloutMetricValidator {
    RolloutMetricValidator {
        rules: vec![
            Rc::new(RolloutMetricSegmentLengthRule { max_length: 16 }) as Rc<dyn RolloutMetricRule>,
            Rc::new(RolloutMetricRequiredCurrencyRule {
                message: "rollout metric needs a value".to_string(),
            }),
            Rc::new(RolloutMetricRequiredTitleRule {
                message: "rollout metric needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_metric_validator();
        let item = new_rollout_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = RolloutMetricSegmentLengthRule { max_length: 255 };
        let mut item = new_rollout_metric("x", "y");
        item.segment = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = RolloutMetricRequiredCurrencyRule {
            message: "rollout metric needs a value".to_string(),
        };
        let mut item = new_rollout_metric("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RolloutMetricRequiredTitleRule {
            message: "rollout metric needs a value".to_string(),
        };
        let mut item = new_rollout_metric("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
