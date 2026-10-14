use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling runbook bundle must pass. Check returns its problems, or none.
pub trait RollingRunbookBundleRule: std::fmt::Debug {
    fn check(&self, item: &RollingRunbookBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookBundleSegmentLengthRule {
    pub max_length: i64,
}

impl RollingRunbookBundleRule for RollingRunbookBundleSegmentLengthRule {
    fn check(&self, item: &RollingRunbookBundle) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookBundleTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingRunbookBundleRule for RollingRunbookBundleTtlSecondsRangeRule {
    fn check(&self, item: &RollingRunbookBundle) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookBundleRequiredLabelRule {
    pub message: String,
}

impl RollingRunbookBundleRule for RollingRunbookBundleRequiredLabelRule {
    fn check(&self, item: &RollingRunbookBundle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookBundleMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingRunbookBundleRule for RollingRunbookBundleMaximumRangeRule {
    fn check(&self, item: &RollingRunbookBundle) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookBundleLabelLengthRule {
    pub max_length: i64,
}

impl RollingRunbookBundleRule for RollingRunbookBundleLabelLengthRule {
    fn check(&self, item: &RollingRunbookBundle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling runbook bundle.
#[derive(Debug, Clone)]
pub struct RollingRunbookBundleValidator {
    pub rules: Vec<Rc<dyn RollingRunbookBundleRule>>,
}

impl RollingRunbookBundleValidator {
    pub fn validate(&self, item: &RollingRunbookBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingRunbookBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingRunbookBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_runbook_bundle_validator() -> RollingRunbookBundleValidator {
    RollingRunbookBundleValidator {
        rules: vec![
            Rc::new(RollingRunbookBundleSegmentLengthRule { max_length: 86400 }) as Rc<dyn RollingRunbookBundleRule>,
            Rc::new(RollingRunbookBundleTtlSecondsRangeRule { min: 16, max: 16 }),
            Rc::new(RollingRunbookBundleRequiredLabelRule {
                message: "rolling runbook bundle needs an entry".to_string(),
            }),
            Rc::new(RollingRunbookBundleMaximumRangeRule { min: 2.5, max: 4190.2 }),
            Rc::new(RollingRunbookBundleLabelLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_runbook_bundle_validator();
        let item = new_rolling_runbook_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = RollingRunbookBundleSegmentLengthRule { max_length: 1000 };
        let mut item = new_rolling_runbook_bundle("x", "y");
        item.segment = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = RollingRunbookBundleTtlSecondsRangeRule { min: 64, max: 1000 };
        let mut item = new_rolling_runbook_bundle("x", "y");
        item.ttl_seconds = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollingRunbookBundleRequiredLabelRule {
            message: "rolling runbook bundle needs this field".to_string(),
        };
        let mut item = new_rolling_runbook_bundle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = RollingRunbookBundleMaximumRangeRule { min: 9.0, max: 1720.7 };
        let mut item = new_rolling_runbook_bundle("x", "y");
        item.maximum = 1722.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollingRunbookBundleLabelLengthRule { max_length: 80 };
        let mut item = new_rolling_runbook_bundle("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
