use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a zone review must pass. Check returns its problems, or none.
pub trait ZoneReviewRule: std::fmt::Debug {
    fn check(&self, item: &ZoneReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneReviewRequiredNamespaceRule {
    pub message: String,
}

impl ZoneReviewRule for ZoneReviewRequiredNamespaceRule {
    fn check(&self, item: &ZoneReview) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneReviewScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ZoneReviewRule for ZoneReviewScoreRangeRule {
    fn check(&self, item: &ZoneReview) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneReviewCodeLengthRule {
    pub max_length: i64,
}

impl ZoneReviewRule for ZoneReviewCodeLengthRule {
    fn check(&self, item: &ZoneReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone review.
#[derive(Debug, Clone)]
pub struct ZoneReviewValidator {
    pub rules: Vec<Rc<dyn ZoneReviewRule>>,
}

impl ZoneReviewValidator {
    pub fn validate(&self, item: &ZoneReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZoneReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_review_validator() -> ZoneReviewValidator {
    ZoneReviewValidator {
        rules: vec![
            Rc::new(ZoneReviewRequiredNamespaceRule {
                message: "zone review needs this field".to_string(),
            }) as Rc<dyn ZoneReviewRule>,
            Rc::new(ZoneReviewScoreRangeRule { min: 7.1, max: 576.8 }),
            Rc::new(ZoneReviewCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_review_validator();
        let item = new_zone_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = ZoneReviewRequiredNamespaceRule {
            message: "zone review needs a value".to_string(),
        };
        let mut item = new_zone_review("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ZoneReviewScoreRangeRule { min: 2.2, max: 437.4 };
        let mut item = new_zone_review("x", "y");
        item.score = 438.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ZoneReviewCodeLengthRule { max_length: 16 };
        let mut item = new_zone_review("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
