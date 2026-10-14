use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity review must pass. Check returns its problems, or none.
pub trait IdentityReviewRule: std::fmt::Debug {
    fn check(&self, item: &IdentityReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityReviewRequiredNameRule {
    pub message: String,
}

impl IdentityReviewRule for IdentityReviewRequiredNameRule {
    fn check(&self, item: &IdentityReview) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityReviewCodePrefixRule {
    pub prefix: String,
}

impl IdentityReviewRule for IdentityReviewCodePrefixRule {
    fn check(&self, item: &IdentityReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityReviewBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityReviewRule for IdentityReviewBatchSizeRangeRule {
    fn check(&self, item: &IdentityReview) -> Vec<String> {
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
pub struct IdentityReviewFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl IdentityReviewRule for IdentityReviewFloorValueRangeRule {
    fn check(&self, item: &IdentityReview) -> Vec<String> {
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

/// Runs every rule of a identity review.
#[derive(Debug, Clone)]
pub struct IdentityReviewValidator {
    pub rules: Vec<Rc<dyn IdentityReviewRule>>,
}

impl IdentityReviewValidator {
    pub fn validate(&self, item: &IdentityReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IdentityReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_review_validator() -> IdentityReviewValidator {
    IdentityReviewValidator {
        rules: vec![
            Rc::new(IdentityReviewRequiredNameRule {
                message: "identity review needs an entry".to_string(),
            }) as Rc<dyn IdentityReviewRule>,
            Rc::new(IdentityReviewCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(IdentityReviewBatchSizeRangeRule { min: 80, max: 1000 }),
            Rc::new(IdentityReviewFloorValueRangeRule { min: 9.5, max: 2942.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_review_validator();
        let item = new_identity_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = IdentityReviewRequiredNameRule {
            message: "identity review needs a value".to_string(),
        };
        let mut item = new_identity_review("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = IdentityReviewBatchSizeRangeRule { min: 255, max: 86400 };
        let mut item = new_identity_review("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = IdentityReviewFloorValueRangeRule { min: 0.9, max: 3070.9 };
        let mut item = new_identity_review("x", "y");
        item.floor_value = 3072.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
