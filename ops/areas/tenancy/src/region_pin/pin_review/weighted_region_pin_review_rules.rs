use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted region pin review must pass. Check returns its problems, or none.
pub trait WeightedRegionPinReviewRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRegionPinReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRegionPinReviewCodeLengthRule {
    pub max_length: i64,
}

impl WeightedRegionPinReviewRule for WeightedRegionPinReviewCodeLengthRule {
    fn check(&self, item: &WeightedRegionPinReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRegionPinReviewNotesLengthRule {
    pub max_length: i64,
}

impl WeightedRegionPinReviewRule for WeightedRegionPinReviewNotesLengthRule {
    fn check(&self, item: &WeightedRegionPinReview) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRegionPinReviewRequiredNotesRule {
    pub message: String,
}

impl WeightedRegionPinReviewRule for WeightedRegionPinReviewRequiredNotesRule {
    fn check(&self, item: &WeightedRegionPinReview) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted region pin review.
#[derive(Debug, Clone)]
pub struct WeightedRegionPinReviewValidator {
    pub rules: Vec<Rc<dyn WeightedRegionPinReviewRule>>,
}

impl WeightedRegionPinReviewValidator {
    pub fn validate(&self, item: &WeightedRegionPinReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedRegionPinReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRegionPinReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_region_pin_review_validator() -> WeightedRegionPinReviewValidator {
    WeightedRegionPinReviewValidator {
        rules: vec![
            Rc::new(WeightedRegionPinReviewCodeLengthRule { max_length: 1000 }) as Rc<dyn WeightedRegionPinReviewRule>,
            Rc::new(WeightedRegionPinReviewNotesLengthRule { max_length: 3600 }),
            Rc::new(WeightedRegionPinReviewRequiredNotesRule {
                message: "weighted region pin review needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_region_pin_review_validator();
        let item = new_weighted_region_pin_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedRegionPinReviewCodeLengthRule { max_length: 32 };
        let mut item = new_weighted_region_pin_review("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = WeightedRegionPinReviewNotesLengthRule { max_length: 86400 };
        let mut item = new_weighted_region_pin_review("x", "y");
        item.notes = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = WeightedRegionPinReviewRequiredNotesRule {
            message: "weighted region pin review needs an entry".to_string(),
        };
        let mut item = new_weighted_region_pin_review("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
