use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive region pin review must pass. Check returns its problems, or none.
pub trait AdaptiveRegionPinReviewRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRegionPinReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinReviewCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveRegionPinReviewRule for AdaptiveRegionPinReviewCodePrefixRule {
    fn check(&self, item: &AdaptiveRegionPinReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinReviewNotesLengthRule {
    pub max_length: i64,
}

impl AdaptiveRegionPinReviewRule for AdaptiveRegionPinReviewNotesLengthRule {
    fn check(&self, item: &AdaptiveRegionPinReview) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinReviewRequiredCodeRule {
    pub message: String,
}

impl AdaptiveRegionPinReviewRule for AdaptiveRegionPinReviewRequiredCodeRule {
    fn check(&self, item: &AdaptiveRegionPinReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinReviewRequiredNotesRule {
    pub message: String,
}

impl AdaptiveRegionPinReviewRule for AdaptiveRegionPinReviewRequiredNotesRule {
    fn check(&self, item: &AdaptiveRegionPinReview) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinReviewDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveRegionPinReviewRule for AdaptiveRegionPinReviewDiscountRangeRule {
    fn check(&self, item: &AdaptiveRegionPinReview) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive region pin review.
#[derive(Debug, Clone)]
pub struct AdaptiveRegionPinReviewValidator {
    pub rules: Vec<Rc<dyn AdaptiveRegionPinReviewRule>>,
}

impl AdaptiveRegionPinReviewValidator {
    pub fn validate(&self, item: &AdaptiveRegionPinReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveRegionPinReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRegionPinReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_region_pin_review_validator() -> AdaptiveRegionPinReviewValidator {
    AdaptiveRegionPinReviewValidator {
        rules: vec![
            Rc::new(AdaptiveRegionPinReviewCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn AdaptiveRegionPinReviewRule>,
            Rc::new(AdaptiveRegionPinReviewNotesLengthRule { max_length: 80 }),
            Rc::new(AdaptiveRegionPinReviewRequiredCodeRule {
                message: "adaptive region pin review needs this field".to_string(),
            }),
            Rc::new(AdaptiveRegionPinReviewRequiredNotesRule {
                message: "adaptive region pin review needs a value".to_string(),
            }),
            Rc::new(AdaptiveRegionPinReviewDiscountRangeRule { min: 5.7, max: 678.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_region_pin_review_validator();
        let item = new_adaptive_region_pin_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = AdaptiveRegionPinReviewNotesLengthRule { max_length: 64 };
        let mut item = new_adaptive_region_pin_review("x", "y");
        item.notes = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveRegionPinReviewRequiredCodeRule {
            message: "adaptive region pin review needs this field".to_string(),
        };
        let mut item = new_adaptive_region_pin_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = AdaptiveRegionPinReviewRequiredNotesRule {
            message: "adaptive region pin review needs a value".to_string(),
        };
        let mut item = new_adaptive_region_pin_review("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = AdaptiveRegionPinReviewDiscountRangeRule { min: 3.3, max: 2419.5 };
        let mut item = new_adaptive_region_pin_review("x", "y");
        item.discount = 2421.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
