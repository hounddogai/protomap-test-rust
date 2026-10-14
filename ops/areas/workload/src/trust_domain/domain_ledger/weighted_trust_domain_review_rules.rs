use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted trust domain review must pass. Check returns its problems, or none.
pub trait WeightedTrustDomainReviewRule: std::fmt::Debug {
    fn check(&self, item: &WeightedTrustDomainReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainReviewRequiredTitleRule {
    pub message: String,
}

impl WeightedTrustDomainReviewRule for WeightedTrustDomainReviewRequiredTitleRule {
    fn check(&self, item: &WeightedTrustDomainReview) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainReviewRequiredCodeRule {
    pub message: String,
}

impl WeightedTrustDomainReviewRule for WeightedTrustDomainReviewRequiredCodeRule {
    fn check(&self, item: &WeightedTrustDomainReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainReviewAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedTrustDomainReviewRule for WeightedTrustDomainReviewAttemptsRangeRule {
    fn check(&self, item: &WeightedTrustDomainReview) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainReviewCodeLengthRule {
    pub max_length: i64,
}

impl WeightedTrustDomainReviewRule for WeightedTrustDomainReviewCodeLengthRule {
    fn check(&self, item: &WeightedTrustDomainReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainReviewDescriptionLengthRule {
    pub max_length: i64,
}

impl WeightedTrustDomainReviewRule for WeightedTrustDomainReviewDescriptionLengthRule {
    fn check(&self, item: &WeightedTrustDomainReview) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainReviewRequiredRegionRule {
    pub message: String,
}

impl WeightedTrustDomainReviewRule for WeightedTrustDomainReviewRequiredRegionRule {
    fn check(&self, item: &WeightedTrustDomainReview) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted trust domain review.
#[derive(Debug, Clone)]
pub struct WeightedTrustDomainReviewValidator {
    pub rules: Vec<Rc<dyn WeightedTrustDomainReviewRule>>,
}

impl WeightedTrustDomainReviewValidator {
    pub fn validate(&self, item: &WeightedTrustDomainReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedTrustDomainReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedTrustDomainReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_trust_domain_review_validator() -> WeightedTrustDomainReviewValidator {
    WeightedTrustDomainReviewValidator {
        rules: vec![
            Rc::new(WeightedTrustDomainReviewRequiredTitleRule {
                message: "weighted trust domain review needs an entry".to_string(),
            }) as Rc<dyn WeightedTrustDomainReviewRule>,
            Rc::new(WeightedTrustDomainReviewRequiredCodeRule {
                message: "weighted trust domain review needs a value".to_string(),
            }),
            Rc::new(WeightedTrustDomainReviewAttemptsRangeRule { min: 120, max: 86400 }),
            Rc::new(WeightedTrustDomainReviewCodeLengthRule { max_length: 80 }),
            Rc::new(WeightedTrustDomainReviewDescriptionLengthRule { max_length: 86400 }),
            Rc::new(WeightedTrustDomainReviewRequiredRegionRule {
                message: "weighted trust domain review needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_trust_domain_review_validator();
        let item = new_weighted_trust_domain_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = WeightedTrustDomainReviewRequiredTitleRule {
            message: "weighted trust domain review needs an entry".to_string(),
        };
        let mut item = new_weighted_trust_domain_review("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedTrustDomainReviewRequiredCodeRule {
            message: "weighted trust domain review needs a value".to_string(),
        };
        let mut item = new_weighted_trust_domain_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = WeightedTrustDomainReviewAttemptsRangeRule { min: 3600, max: 86400 };
        let mut item = new_weighted_trust_domain_review("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedTrustDomainReviewCodeLengthRule { max_length: 32 };
        let mut item = new_weighted_trust_domain_review("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = WeightedTrustDomainReviewDescriptionLengthRule { max_length: 64 };
        let mut item = new_weighted_trust_domain_review("x", "y");
        item.description = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = WeightedTrustDomainReviewRequiredRegionRule {
            message: "weighted trust domain review needs an entry".to_string(),
        };
        let mut item = new_weighted_trust_domain_review("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
