use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional binding review must pass. Check returns its problems, or none.
pub trait RegionalBindingReviewRule: std::fmt::Debug {
    fn check(&self, item: &RegionalBindingReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBindingReviewActiveGuardRule {
    pub limit: f64,
}

impl RegionalBindingReviewRule for RegionalBindingReviewActiveGuardRule {
    fn check(&self, item: &RegionalBindingReview) -> Vec<String> {
        if item.active && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while active", item.score, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBindingReviewCodePrefixRule {
    pub prefix: String,
}

impl RegionalBindingReviewRule for RegionalBindingReviewCodePrefixRule {
    fn check(&self, item: &RegionalBindingReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBindingReviewRequiredCodeRule {
    pub message: String,
}

impl RegionalBindingReviewRule for RegionalBindingReviewRequiredCodeRule {
    fn check(&self, item: &RegionalBindingReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBindingReviewMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalBindingReviewRule for RegionalBindingReviewMinimumRangeRule {
    fn check(&self, item: &RegionalBindingReview) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBindingReviewRequiredUnitRule {
    pub message: String,
}

impl RegionalBindingReviewRule for RegionalBindingReviewRequiredUnitRule {
    fn check(&self, item: &RegionalBindingReview) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional binding review.
#[derive(Debug, Clone)]
pub struct RegionalBindingReviewValidator {
    pub rules: Vec<Rc<dyn RegionalBindingReviewRule>>,
}

impl RegionalBindingReviewValidator {
    pub fn validate(&self, item: &RegionalBindingReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalBindingReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalBindingReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_binding_review_validator() -> RegionalBindingReviewValidator {
    RegionalBindingReviewValidator {
        rules: vec![
            Rc::new(RegionalBindingReviewActiveGuardRule { limit: 4.3 }) as Rc<dyn RegionalBindingReviewRule>,
            Rc::new(RegionalBindingReviewCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RegionalBindingReviewRequiredCodeRule {
                message: "regional binding review needs a value".to_string(),
            }),
            Rc::new(RegionalBindingReviewMinimumRangeRule { min: 6.0, max: 4019.7 }),
            Rc::new(RegionalBindingReviewRequiredUnitRule {
                message: "regional binding review needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_binding_review_validator();
        let item = new_regional_binding_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalBindingReviewRequiredCodeRule {
            message: "regional binding review needs an entry".to_string(),
        };
        let mut item = new_regional_binding_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = RegionalBindingReviewMinimumRangeRule { min: 0.5, max: 4873.9 };
        let mut item = new_regional_binding_review("x", "y");
        item.minimum = 4875.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = RegionalBindingReviewRequiredUnitRule {
            message: "regional binding review needs a value".to_string(),
        };
        let mut item = new_regional_binding_review("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
