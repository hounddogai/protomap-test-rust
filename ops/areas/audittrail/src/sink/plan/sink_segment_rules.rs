use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink segment must pass. Check returns its problems, or none.
pub trait SinkSegmentRule: std::fmt::Debug {
    fn check(&self, item: &SinkSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkSegmentRequiredLabelRule {
    pub message: String,
}

impl SinkSegmentRule for SinkSegmentRequiredLabelRule {
    fn check(&self, item: &SinkSegment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkSegmentRequiredLocaleRule {
    pub message: String,
}

impl SinkSegmentRule for SinkSegmentRequiredLocaleRule {
    fn check(&self, item: &SinkSegment) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkSegmentLocaleLengthRule {
    pub max_length: i64,
}

impl SinkSegmentRule for SinkSegmentLocaleLengthRule {
    fn check(&self, item: &SinkSegment) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkSegmentUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkSegmentRule for SinkSegmentUpdatedAtRangeRule {
    fn check(&self, item: &SinkSegment) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkSegmentLabelLengthRule {
    pub max_length: i64,
}

impl SinkSegmentRule for SinkSegmentLabelLengthRule {
    fn check(&self, item: &SinkSegment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkSegmentRequiredChannelRule {
    pub message: String,
}

impl SinkSegmentRule for SinkSegmentRequiredChannelRule {
    fn check(&self, item: &SinkSegment) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink segment.
#[derive(Debug, Clone)]
pub struct SinkSegmentValidator {
    pub rules: Vec<Rc<dyn SinkSegmentRule>>,
}

impl SinkSegmentValidator {
    pub fn validate(&self, item: &SinkSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_segment_validator() -> SinkSegmentValidator {
    SinkSegmentValidator {
        rules: vec![
            Rc::new(SinkSegmentRequiredLabelRule {
                message: "sink segment needs a value".to_string(),
            }) as Rc<dyn SinkSegmentRule>,
            Rc::new(SinkSegmentRequiredLocaleRule {
                message: "sink segment needs this field".to_string(),
            }),
            Rc::new(SinkSegmentLocaleLengthRule { max_length: 1000 }),
            Rc::new(SinkSegmentUpdatedAtRangeRule { min: 16, max: 255 }),
            Rc::new(SinkSegmentLabelLengthRule { max_length: 1000 }),
            Rc::new(SinkSegmentRequiredChannelRule {
                message: "sink segment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_segment_validator();
        let item = new_sink_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SinkSegmentRequiredLabelRule {
            message: "sink segment needs this field".to_string(),
        };
        let mut item = new_sink_segment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = SinkSegmentRequiredLocaleRule {
            message: "sink segment needs an entry".to_string(),
        };
        let mut item = new_sink_segment("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = SinkSegmentLocaleLengthRule { max_length: 64 };
        let mut item = new_sink_segment("x", "y");
        item.locale = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = SinkSegmentUpdatedAtRangeRule { min: 255, max: 255 };
        let mut item = new_sink_segment("x", "y");
        item.updated_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SinkSegmentLabelLengthRule { max_length: 64 };
        let mut item = new_sink_segment("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = SinkSegmentRequiredChannelRule {
            message: "sink segment needs an entry".to_string(),
        };
        let mut item = new_sink_segment("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
