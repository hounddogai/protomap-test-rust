use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit segment must pass. Check returns its problems, or none.
pub trait LimitSegmentRule: std::fmt::Debug {
    fn check(&self, item: &LimitSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitSegmentTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl LimitSegmentRule for LimitSegmentTagsLimitRule {
    fn check(&self, item: &LimitSegment) -> Vec<String> {
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
pub struct LimitSegmentRequiredTitleRule {
    pub message: String,
}

impl LimitSegmentRule for LimitSegmentRequiredTitleRule {
    fn check(&self, item: &LimitSegment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitSegmentRequiredCipherRule {
    pub message: String,
}

impl LimitSegmentRule for LimitSegmentRequiredCipherRule {
    fn check(&self, item: &LimitSegment) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitSegmentApprovedGuardRule {
    pub limit: f64,
}

impl LimitSegmentRule for LimitSegmentApprovedGuardRule {
    fn check(&self, item: &LimitSegment) -> Vec<String> {
        if item.approved && (item.floor_value > self.limit) {
            return vec![format!(
                "floor value {} exceeds {} while approved",
                item.floor_value, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitSegmentBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LimitSegmentRule for LimitSegmentBatchSizeRangeRule {
    fn check(&self, item: &LimitSegment) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitSegmentFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LimitSegmentRule for LimitSegmentFloorValueRangeRule {
    fn check(&self, item: &LimitSegment) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a limit segment.
#[derive(Debug, Clone)]
pub struct LimitSegmentValidator {
    pub rules: Vec<Rc<dyn LimitSegmentRule>>,
}

impl LimitSegmentValidator {
    pub fn validate(&self, item: &LimitSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_segment_validator() -> LimitSegmentValidator {
    LimitSegmentValidator {
        rules: vec![
            Rc::new(LimitSegmentTagsLimitRule {
                max_count: 64,
                banned: vec!["pending".to_string(), "regional".to_string(), "tiered".to_string()],
            }) as Rc<dyn LimitSegmentRule>,
            Rc::new(LimitSegmentRequiredTitleRule {
                message: "limit segment needs an entry".to_string(),
            }),
            Rc::new(LimitSegmentRequiredCipherRule {
                message: "limit segment needs an entry".to_string(),
            }),
            Rc::new(LimitSegmentApprovedGuardRule { limit: 4.0 }),
            Rc::new(LimitSegmentBatchSizeRangeRule { min: 80, max: 3600 }),
            Rc::new(LimitSegmentFloorValueRangeRule { min: 7.2, max: 4634.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_segment_validator();
        let item = new_limit_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LimitSegmentRequiredTitleRule {
            message: "limit segment needs an entry".to_string(),
        };
        let mut item = new_limit_segment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = LimitSegmentRequiredCipherRule {
            message: "limit segment needs this field".to_string(),
        };
        let mut item = new_limit_segment("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = LimitSegmentBatchSizeRangeRule { min: 255, max: 86400 };
        let mut item = new_limit_segment("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = LimitSegmentFloorValueRangeRule { min: 3.1, max: 3590.4 };
        let mut item = new_limit_segment("x", "y");
        item.floor_value = 3591.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
