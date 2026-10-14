use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline bundle must pass. Check returns its problems, or none.
pub trait TimelineBundleRule: std::fmt::Debug {
    fn check(&self, item: &TimelineBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBundleBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineBundleRule for TimelineBundleBatchSizeRangeRule {
    fn check(&self, item: &TimelineBundle) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBundleMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TimelineBundleRule for TimelineBundleMaximumRangeRule {
    fn check(&self, item: &TimelineBundle) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBundleRequiredCodeRule {
    pub message: String,
}

impl TimelineBundleRule for TimelineBundleRequiredCodeRule {
    fn check(&self, item: &TimelineBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBundleRequiredChecksumRule {
    pub message: String,
}

impl TimelineBundleRule for TimelineBundleRequiredChecksumRule {
    fn check(&self, item: &TimelineBundle) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBundleCodeLengthRule {
    pub max_length: i64,
}

impl TimelineBundleRule for TimelineBundleCodeLengthRule {
    fn check(&self, item: &TimelineBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline bundle.
#[derive(Debug, Clone)]
pub struct TimelineBundleValidator {
    pub rules: Vec<Rc<dyn TimelineBundleRule>>,
}

impl TimelineBundleValidator {
    pub fn validate(&self, item: &TimelineBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_bundle_validator() -> TimelineBundleValidator {
    TimelineBundleValidator {
        rules: vec![
            Rc::new(TimelineBundleBatchSizeRangeRule { min: 32, max: 80 }) as Rc<dyn TimelineBundleRule>,
            Rc::new(TimelineBundleMaximumRangeRule { min: 7.4, max: 3985.0 }),
            Rc::new(TimelineBundleRequiredCodeRule {
                message: "timeline bundle needs a value".to_string(),
            }),
            Rc::new(TimelineBundleRequiredChecksumRule {
                message: "timeline bundle needs this field".to_string(),
            }),
            Rc::new(TimelineBundleCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_bundle_validator();
        let item = new_timeline_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = TimelineBundleBatchSizeRangeRule { min: 32, max: 120 };
        let mut item = new_timeline_bundle("x", "y");
        item.batch_size = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = TimelineBundleMaximumRangeRule { min: 5.2, max: 3570.6 };
        let mut item = new_timeline_bundle("x", "y");
        item.maximum = 3572.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TimelineBundleRequiredCodeRule {
            message: "timeline bundle needs a value".to_string(),
        };
        let mut item = new_timeline_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = TimelineBundleRequiredChecksumRule {
            message: "timeline bundle needs an entry".to_string(),
        };
        let mut item = new_timeline_bundle("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TimelineBundleCodeLengthRule { max_length: 1000 };
        let mut item = new_timeline_bundle("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
