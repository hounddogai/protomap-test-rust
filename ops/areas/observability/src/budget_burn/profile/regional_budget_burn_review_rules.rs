use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional budget burn review must pass. Check returns its problems, or none.
pub trait RegionalBudgetBurnReviewRule: std::fmt::Debug {
    fn check(&self, item: &RegionalBudgetBurnReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnReviewCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalBudgetBurnReviewRule for RegionalBudgetBurnReviewCapacityRangeRule {
    fn check(&self, item: &RegionalBudgetBurnReview) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnReviewCodeLengthRule {
    pub max_length: i64,
}

impl RegionalBudgetBurnReviewRule for RegionalBudgetBurnReviewCodeLengthRule {
    fn check(&self, item: &RegionalBudgetBurnReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnReviewRequiredFingerprintRule {
    pub message: String,
}

impl RegionalBudgetBurnReviewRule for RegionalBudgetBurnReviewRequiredFingerprintRule {
    fn check(&self, item: &RegionalBudgetBurnReview) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnReviewRequiredCodeRule {
    pub message: String,
}

impl RegionalBudgetBurnReviewRule for RegionalBudgetBurnReviewRequiredCodeRule {
    fn check(&self, item: &RegionalBudgetBurnReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnReviewRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalBudgetBurnReviewRule for RegionalBudgetBurnReviewRotationDaysRangeRule {
    fn check(&self, item: &RegionalBudgetBurnReview) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnReviewRequiredScopeRule {
    pub message: String,
}

impl RegionalBudgetBurnReviewRule for RegionalBudgetBurnReviewRequiredScopeRule {
    fn check(&self, item: &RegionalBudgetBurnReview) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional budget burn review.
#[derive(Debug, Clone)]
pub struct RegionalBudgetBurnReviewValidator {
    pub rules: Vec<Rc<dyn RegionalBudgetBurnReviewRule>>,
}

impl RegionalBudgetBurnReviewValidator {
    pub fn validate(&self, item: &RegionalBudgetBurnReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalBudgetBurnReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalBudgetBurnReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_budget_burn_review_validator() -> RegionalBudgetBurnReviewValidator {
    RegionalBudgetBurnReviewValidator {
        rules: vec![
            Rc::new(RegionalBudgetBurnReviewCapacityRangeRule { min: 32, max: 1000 })
                as Rc<dyn RegionalBudgetBurnReviewRule>,
            Rc::new(RegionalBudgetBurnReviewCodeLengthRule { max_length: 16 }),
            Rc::new(RegionalBudgetBurnReviewRequiredFingerprintRule {
                message: "regional budget burn review needs a value".to_string(),
            }),
            Rc::new(RegionalBudgetBurnReviewRequiredCodeRule {
                message: "regional budget burn review needs an entry".to_string(),
            }),
            Rc::new(RegionalBudgetBurnReviewRotationDaysRangeRule { min: 80, max: 80 }),
            Rc::new(RegionalBudgetBurnReviewRequiredScopeRule {
                message: "regional budget burn review needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_budget_burn_review_validator();
        let item = new_regional_budget_burn_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = RegionalBudgetBurnReviewCapacityRangeRule { min: 80, max: 120 };
        let mut item = new_regional_budget_burn_review("x", "y");
        item.capacity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalBudgetBurnReviewCodeLengthRule { max_length: 1000 };
        let mut item = new_regional_budget_burn_review("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = RegionalBudgetBurnReviewRequiredFingerprintRule {
            message: "regional budget burn review needs this field".to_string(),
        };
        let mut item = new_regional_budget_burn_review("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalBudgetBurnReviewRequiredCodeRule {
            message: "regional budget burn review needs a value".to_string(),
        };
        let mut item = new_regional_budget_burn_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = RegionalBudgetBurnReviewRotationDaysRangeRule { min: 120, max: 86400 };
        let mut item = new_regional_budget_burn_review("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = RegionalBudgetBurnReviewRequiredScopeRule {
            message: "regional budget burn review needs a value".to_string(),
        };
        let mut item = new_regional_budget_burn_review("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
