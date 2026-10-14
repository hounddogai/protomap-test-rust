use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive retention metric must pass. Check returns its problems, or none.
pub trait AdaptiveRetentionMetricRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRetentionMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRetentionMetricTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl AdaptiveRetentionMetricRule for AdaptiveRetentionMetricTagsLimitRule {
    fn check(&self, item: &AdaptiveRetentionMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRetentionMetricSegmentLengthRule {
    pub max_length: i64,
}

impl AdaptiveRetentionMetricRule for AdaptiveRetentionMetricSegmentLengthRule {
    fn check(&self, item: &AdaptiveRetentionMetric) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRetentionMetricAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveRetentionMetricRule for AdaptiveRetentionMetricAmountRangeRule {
    fn check(&self, item: &AdaptiveRetentionMetric) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a adaptive retention metric.
#[derive(Debug, Clone)]
pub struct AdaptiveRetentionMetricValidator {
    pub rules: Vec<Rc<dyn AdaptiveRetentionMetricRule>>,
}

impl AdaptiveRetentionMetricValidator {
    pub fn validate(&self, item: &AdaptiveRetentionMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveRetentionMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRetentionMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_retention_metric_validator() -> AdaptiveRetentionMetricValidator {
    AdaptiveRetentionMetricValidator {
        rules: vec![
            Rc::new(AdaptiveRetentionMetricTagsLimitRule {
                max_count: 64,
                banned: vec!["adaptive".to_string(), "manual".to_string()],
            }) as Rc<dyn AdaptiveRetentionMetricRule>,
            Rc::new(AdaptiveRetentionMetricSegmentLengthRule { max_length: 32 }),
            Rc::new(AdaptiveRetentionMetricAmountRangeRule { min: 3.0, max: 1618.2 }),
        ],
    }
}
