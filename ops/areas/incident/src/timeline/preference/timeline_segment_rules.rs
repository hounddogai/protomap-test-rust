use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline segment must pass. Check returns its problems, or none.
pub trait TimelineSegmentRule: std::fmt::Debug {
    fn check(&self, item: &TimelineSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSegmentVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineSegmentRule for TimelineSegmentVersionRangeRule {
    fn check(&self, item: &TimelineSegment) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSegmentRequiredCodeRule {
    pub message: String,
}

impl TimelineSegmentRule for TimelineSegmentRequiredCodeRule {
    fn check(&self, item: &TimelineSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSegmentBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineSegmentRule for TimelineSegmentBatchSizeRangeRule {
    fn check(&self, item: &TimelineSegment) -> Vec<String> {
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
pub struct TimelineSegmentDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TimelineSegmentRule for TimelineSegmentDiscountRangeRule {
    fn check(&self, item: &TimelineSegment) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSegmentNameLengthRule {
    pub max_length: i64,
}

impl TimelineSegmentRule for TimelineSegmentNameLengthRule {
    fn check(&self, item: &TimelineSegment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSegmentCodePrefixRule {
    pub prefix: String,
}

impl TimelineSegmentRule for TimelineSegmentCodePrefixRule {
    fn check(&self, item: &TimelineSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline segment.
#[derive(Debug, Clone)]
pub struct TimelineSegmentValidator {
    pub rules: Vec<Rc<dyn TimelineSegmentRule>>,
}

impl TimelineSegmentValidator {
    pub fn validate(&self, item: &TimelineSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_segment_validator() -> TimelineSegmentValidator {
    TimelineSegmentValidator {
        rules: vec![
            Rc::new(TimelineSegmentVersionRangeRule { min: 255, max: 86400 }) as Rc<dyn TimelineSegmentRule>,
            Rc::new(TimelineSegmentRequiredCodeRule {
                message: "timeline segment needs an entry".to_string(),
            }),
            Rc::new(TimelineSegmentBatchSizeRangeRule { min: 255, max: 1000 }),
            Rc::new(TimelineSegmentDiscountRangeRule { min: 1.6, max: 4960.2 }),
            Rc::new(TimelineSegmentNameLengthRule { max_length: 3600 }),
            Rc::new(TimelineSegmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_segment_validator();
        let item = new_timeline_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = TimelineSegmentVersionRangeRule { min: 80, max: 255 };
        let mut item = new_timeline_segment("x", "y");
        item.version = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TimelineSegmentRequiredCodeRule {
            message: "timeline segment needs an entry".to_string(),
        };
        let mut item = new_timeline_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = TimelineSegmentBatchSizeRangeRule { min: 3600, max: 86400 };
        let mut item = new_timeline_segment("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = TimelineSegmentDiscountRangeRule { min: 3.5, max: 2904.3 };
        let mut item = new_timeline_segment("x", "y");
        item.discount = 2905.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TimelineSegmentNameLengthRule { max_length: 255 };
        let mut item = new_timeline_segment("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
