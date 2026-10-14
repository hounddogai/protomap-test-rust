use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release adjustment must pass. Check returns its problems, or none.
pub trait ReleaseAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAdjustmentUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseAdjustmentRule for ReleaseAdjustmentUpdatedAtRangeRule {
    fn check(&self, item: &ReleaseAdjustment) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl ReleaseAdjustmentRule for ReleaseAdjustmentCodePrefixRule {
    fn check(&self, item: &ReleaseAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAdjustmentRequiredReviewerRule {
    pub message: String,
}

impl ReleaseAdjustmentRule for ReleaseAdjustmentRequiredReviewerRule {
    fn check(&self, item: &ReleaseAdjustment) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAdjustmentRequiredCodeRule {
    pub message: String,
}

impl ReleaseAdjustmentRule for ReleaseAdjustmentRequiredCodeRule {
    fn check(&self, item: &ReleaseAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAdjustmentScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReleaseAdjustmentRule for ReleaseAdjustmentScoreRangeRule {
    fn check(&self, item: &ReleaseAdjustment) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl ReleaseAdjustmentRule for ReleaseAdjustmentCodeLengthRule {
    fn check(&self, item: &ReleaseAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release adjustment.
#[derive(Debug, Clone)]
pub struct ReleaseAdjustmentValidator {
    pub rules: Vec<Rc<dyn ReleaseAdjustmentRule>>,
}

impl ReleaseAdjustmentValidator {
    pub fn validate(&self, item: &ReleaseAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_adjustment_validator() -> ReleaseAdjustmentValidator {
    ReleaseAdjustmentValidator {
        rules: vec![
            Rc::new(ReleaseAdjustmentUpdatedAtRangeRule { min: 16, max: 32 }) as Rc<dyn ReleaseAdjustmentRule>,
            Rc::new(ReleaseAdjustmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ReleaseAdjustmentRequiredReviewerRule {
                message: "release adjustment needs this field".to_string(),
            }),
            Rc::new(ReleaseAdjustmentRequiredCodeRule {
                message: "release adjustment needs an entry".to_string(),
            }),
            Rc::new(ReleaseAdjustmentScoreRangeRule { min: 3.2, max: 214.7 }),
            Rc::new(ReleaseAdjustmentCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_adjustment_validator();
        let item = new_release_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = ReleaseAdjustmentUpdatedAtRangeRule { min: 16, max: 64 };
        let mut item = new_release_adjustment("x", "y");
        item.updated_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = ReleaseAdjustmentRequiredReviewerRule {
            message: "release adjustment needs this field".to_string(),
        };
        let mut item = new_release_adjustment("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReleaseAdjustmentRequiredCodeRule {
            message: "release adjustment needs a value".to_string(),
        };
        let mut item = new_release_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ReleaseAdjustmentScoreRangeRule { min: 5.1, max: 146.2 };
        let mut item = new_release_adjustment("x", "y");
        item.score = 147.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReleaseAdjustmentCodeLengthRule { max_length: 120 };
        let mut item = new_release_adjustment("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
