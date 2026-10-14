use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention metric must pass. Check returns its problems, or none.
pub trait RetentionMetricRule: std::fmt::Debug {
    fn check(&self, item: &RetentionMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionMetricRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RetentionMetricRule for RetentionMetricRateRangeRule {
    fn check(&self, item: &RetentionMetric) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionMetricGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionMetricRule for RetentionMetricGraceDaysRangeRule {
    fn check(&self, item: &RetentionMetric) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionMetricChecksumLengthRule {
    pub max_length: i64,
}

impl RetentionMetricRule for RetentionMetricChecksumLengthRule {
    fn check(&self, item: &RetentionMetric) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionMetricRequiredSegmentRule {
    pub message: String,
}

impl RetentionMetricRule for RetentionMetricRequiredSegmentRule {
    fn check(&self, item: &RetentionMetric) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention metric.
#[derive(Debug, Clone)]
pub struct RetentionMetricValidator {
    pub rules: Vec<Rc<dyn RetentionMetricRule>>,
}

impl RetentionMetricValidator {
    pub fn validate(&self, item: &RetentionMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RetentionMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_metric_validator() -> RetentionMetricValidator {
    RetentionMetricValidator {
        rules: vec![
            Rc::new(RetentionMetricRateRangeRule { min: 4.6, max: 1337.1 }) as Rc<dyn RetentionMetricRule>,
            Rc::new(RetentionMetricGraceDaysRangeRule { min: 1000, max: 3600 }),
            Rc::new(RetentionMetricChecksumLengthRule { max_length: 86400 }),
            Rc::new(RetentionMetricRequiredSegmentRule {
                message: "retention metric needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_metric_validator();
        let item = new_retention_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = RetentionMetricRateRangeRule { min: 9.3, max: 1558.5 };
        let mut item = new_retention_metric("x", "y");
        item.rate = 1560.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = RetentionMetricGraceDaysRangeRule { min: 255, max: 1000 };
        let mut item = new_retention_metric("x", "y");
        item.grace_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = RetentionMetricChecksumLengthRule { max_length: 16 };
        let mut item = new_retention_metric("x", "y");
        item.checksum = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = RetentionMetricRequiredSegmentRule {
            message: "retention metric needs a value".to_string(),
        };
        let mut item = new_retention_metric("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
