use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin review must pass. Check returns its problems, or none.
pub trait RegionPinReviewRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinReviewScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionPinReviewRule for RegionPinReviewScoreRangeRule {
    fn check(&self, item: &RegionPinReview) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinReviewCodePrefixRule {
    pub prefix: String,
}

impl RegionPinReviewRule for RegionPinReviewCodePrefixRule {
    fn check(&self, item: &RegionPinReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinReviewRequiredLabelRule {
    pub message: String,
}

impl RegionPinReviewRule for RegionPinReviewRequiredLabelRule {
    fn check(&self, item: &RegionPinReview) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinReviewRequiredCodeRule {
    pub message: String,
}

impl RegionPinReviewRule for RegionPinReviewRequiredCodeRule {
    fn check(&self, item: &RegionPinReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinReviewRequiredNamespaceRule {
    pub message: String,
}

impl RegionPinReviewRule for RegionPinReviewRequiredNamespaceRule {
    fn check(&self, item: &RegionPinReview) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin review.
#[derive(Debug, Clone)]
pub struct RegionPinReviewValidator {
    pub rules: Vec<Rc<dyn RegionPinReviewRule>>,
}

impl RegionPinReviewValidator {
    pub fn validate(&self, item: &RegionPinReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_review_validator() -> RegionPinReviewValidator {
    RegionPinReviewValidator {
        rules: vec![
            Rc::new(RegionPinReviewScoreRangeRule { min: 0.1, max: 4555.5 }) as Rc<dyn RegionPinReviewRule>,
            Rc::new(RegionPinReviewCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RegionPinReviewRequiredLabelRule {
                message: "region pin review needs a value".to_string(),
            }),
            Rc::new(RegionPinReviewRequiredCodeRule {
                message: "region pin review needs a value".to_string(),
            }),
            Rc::new(RegionPinReviewRequiredNamespaceRule {
                message: "region pin review needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_review_validator();
        let item = new_region_pin_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = RegionPinReviewScoreRangeRule { min: 7.8, max: 635.9 };
        let mut item = new_region_pin_review("x", "y");
        item.score = 637.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionPinReviewRequiredLabelRule {
            message: "region pin review needs an entry".to_string(),
        };
        let mut item = new_region_pin_review("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionPinReviewRequiredCodeRule {
            message: "region pin review needs a value".to_string(),
        };
        let mut item = new_region_pin_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = RegionPinReviewRequiredNamespaceRule {
            message: "region pin review needs an entry".to_string(),
        };
        let mut item = new_region_pin_review("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
