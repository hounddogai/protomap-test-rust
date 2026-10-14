use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary slo segment must pass. Check returns its problems, or none.
pub trait PrimarySloSegmentRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySloSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloSegmentRequiredCodeRule {
    pub message: String,
}

impl PrimarySloSegmentRule for PrimarySloSegmentRequiredCodeRule {
    fn check(&self, item: &PrimarySloSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloSegmentMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimarySloSegmentRule for PrimarySloSegmentMinimumRangeRule {
    fn check(&self, item: &PrimarySloSegment) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloSegmentRequiredTitleRule {
    pub message: String,
}

impl PrimarySloSegmentRule for PrimarySloSegmentRequiredTitleRule {
    fn check(&self, item: &PrimarySloSegment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloSegmentRequiredChannelRule {
    pub message: String,
}

impl PrimarySloSegmentRule for PrimarySloSegmentRequiredChannelRule {
    fn check(&self, item: &PrimarySloSegment) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloSegmentTimezoneLengthRule {
    pub max_length: i64,
}

impl PrimarySloSegmentRule for PrimarySloSegmentTimezoneLengthRule {
    fn check(&self, item: &PrimarySloSegment) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary slo segment.
#[derive(Debug, Clone)]
pub struct PrimarySloSegmentValidator {
    pub rules: Vec<Rc<dyn PrimarySloSegmentRule>>,
}

impl PrimarySloSegmentValidator {
    pub fn validate(&self, item: &PrimarySloSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimarySloSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySloSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_slo_segment_validator() -> PrimarySloSegmentValidator {
    PrimarySloSegmentValidator {
        rules: vec![
            Rc::new(PrimarySloSegmentRequiredCodeRule {
                message: "primary slo segment needs a value".to_string(),
            }) as Rc<dyn PrimarySloSegmentRule>,
            Rc::new(PrimarySloSegmentMinimumRangeRule { min: 2.0, max: 4959.3 }),
            Rc::new(PrimarySloSegmentRequiredTitleRule {
                message: "primary slo segment needs this field".to_string(),
            }),
            Rc::new(PrimarySloSegmentRequiredChannelRule {
                message: "primary slo segment needs an entry".to_string(),
            }),
            Rc::new(PrimarySloSegmentTimezoneLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_slo_segment_validator();
        let item = new_primary_slo_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimarySloSegmentRequiredCodeRule {
            message: "primary slo segment needs an entry".to_string(),
        };
        let mut item = new_primary_slo_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = PrimarySloSegmentMinimumRangeRule { min: 4.0, max: 3319.3 };
        let mut item = new_primary_slo_segment("x", "y");
        item.minimum = 3320.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimarySloSegmentRequiredTitleRule {
            message: "primary slo segment needs a value".to_string(),
        };
        let mut item = new_primary_slo_segment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = PrimarySloSegmentRequiredChannelRule {
            message: "primary slo segment needs a value".to_string(),
        };
        let mut item = new_primary_slo_segment("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = PrimarySloSegmentTimezoneLengthRule { max_length: 120 };
        let mut item = new_primary_slo_segment("x", "y");
        item.timezone = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
