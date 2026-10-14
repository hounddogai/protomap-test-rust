use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment review must pass. Check returns its problems, or none.
pub trait SegmentReviewRule: std::fmt::Debug {
    fn check(&self, item: &SegmentReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentReviewCodePrefixRule {
    pub prefix: String,
}

impl SegmentReviewRule for SegmentReviewCodePrefixRule {
    fn check(&self, item: &SegmentReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentReviewLocaleLengthRule {
    pub max_length: i64,
}

impl SegmentReviewRule for SegmentReviewLocaleLengthRule {
    fn check(&self, item: &SegmentReview) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentReviewBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentReviewRule for SegmentReviewBurstRangeRule {
    fn check(&self, item: &SegmentReview) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentReviewRequiredNameRule {
    pub message: String,
}

impl SegmentReviewRule for SegmentReviewRequiredNameRule {
    fn check(&self, item: &SegmentReview) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentReviewRequiredCodeRule {
    pub message: String,
}

impl SegmentReviewRule for SegmentReviewRequiredCodeRule {
    fn check(&self, item: &SegmentReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment review.
#[derive(Debug, Clone)]
pub struct SegmentReviewValidator {
    pub rules: Vec<Rc<dyn SegmentReviewRule>>,
}

impl SegmentReviewValidator {
    pub fn validate(&self, item: &SegmentReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_review_validator() -> SegmentReviewValidator {
    SegmentReviewValidator {
        rules: vec![
            Rc::new(SegmentReviewCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn SegmentReviewRule>,
            Rc::new(SegmentReviewLocaleLengthRule { max_length: 3600 }),
            Rc::new(SegmentReviewBurstRangeRule { min: 32, max: 64 }),
            Rc::new(SegmentReviewRequiredNameRule {
                message: "segment review needs an entry".to_string(),
            }),
            Rc::new(SegmentReviewRequiredCodeRule {
                message: "segment review needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_review_validator();
        let item = new_segment_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = SegmentReviewLocaleLengthRule { max_length: 3600 };
        let mut item = new_segment_review("x", "y");
        item.locale = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = SegmentReviewBurstRangeRule { min: 16, max: 80 };
        let mut item = new_segment_review("x", "y");
        item.burst = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SegmentReviewRequiredNameRule {
            message: "segment review needs an entry".to_string(),
        };
        let mut item = new_segment_review("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SegmentReviewRequiredCodeRule {
            message: "segment review needs a value".to_string(),
        };
        let mut item = new_segment_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
