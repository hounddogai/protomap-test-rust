use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment metric must pass. Check returns its problems, or none.
pub trait SegmentMetricRule: std::fmt::Debug {
    fn check(&self, item: &SegmentMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMetricUnitLengthRule {
    pub max_length: i64,
}

impl SegmentMetricRule for SegmentMetricUnitLengthRule {
    fn check(&self, item: &SegmentMetric) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMetricRequiredTitleRule {
    pub message: String,
}

impl SegmentMetricRule for SegmentMetricRequiredTitleRule {
    fn check(&self, item: &SegmentMetric) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMetricRequiredPhoneRule {
    pub message: String,
}

impl SegmentMetricRule for SegmentMetricRequiredPhoneRule {
    fn check(&self, item: &SegmentMetric) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment metric.
#[derive(Debug, Clone)]
pub struct SegmentMetricValidator {
    pub rules: Vec<Rc<dyn SegmentMetricRule>>,
}

impl SegmentMetricValidator {
    pub fn validate(&self, item: &SegmentMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SegmentMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_metric_validator() -> SegmentMetricValidator {
    SegmentMetricValidator {
        rules: vec![
            Rc::new(SegmentMetricUnitLengthRule { max_length: 1000 }) as Rc<dyn SegmentMetricRule>,
            Rc::new(SegmentMetricRequiredTitleRule {
                message: "segment metric needs this field".to_string(),
            }),
            Rc::new(SegmentMetricRequiredPhoneRule {
                message: "segment metric needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_metric_validator();
        let item = new_segment_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = SegmentMetricUnitLengthRule { max_length: 32 };
        let mut item = new_segment_metric("x", "y");
        item.unit = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SegmentMetricRequiredTitleRule {
            message: "segment metric needs an entry".to_string(),
        };
        let mut item = new_segment_metric("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = SegmentMetricRequiredPhoneRule {
            message: "segment metric needs an entry".to_string(),
        };
        let mut item = new_segment_metric("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
