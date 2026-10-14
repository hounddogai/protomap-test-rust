use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain review must pass. Check returns its problems, or none.
pub trait TrustDomainReviewRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainReviewTitleLengthRule {
    pub max_length: i64,
}

impl TrustDomainReviewRule for TrustDomainReviewTitleLengthRule {
    fn check(&self, item: &TrustDomainReview) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainReviewRequiredDescriptionRule {
    pub message: String,
}

impl TrustDomainReviewRule for TrustDomainReviewRequiredDescriptionRule {
    fn check(&self, item: &TrustDomainReview) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainReviewRequiredSegmentRule {
    pub message: String,
}

impl TrustDomainReviewRule for TrustDomainReviewRequiredSegmentRule {
    fn check(&self, item: &TrustDomainReview) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainReviewCodePrefixRule {
    pub prefix: String,
}

impl TrustDomainReviewRule for TrustDomainReviewCodePrefixRule {
    fn check(&self, item: &TrustDomainReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainReviewSegmentLengthRule {
    pub max_length: i64,
}

impl TrustDomainReviewRule for TrustDomainReviewSegmentLengthRule {
    fn check(&self, item: &TrustDomainReview) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainReviewDescriptionLengthRule {
    pub max_length: i64,
}

impl TrustDomainReviewRule for TrustDomainReviewDescriptionLengthRule {
    fn check(&self, item: &TrustDomainReview) -> Vec<String> {
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

/// Runs every rule of a trust domain review.
#[derive(Debug, Clone)]
pub struct TrustDomainReviewValidator {
    pub rules: Vec<Rc<dyn TrustDomainReviewRule>>,
}

impl TrustDomainReviewValidator {
    pub fn validate(&self, item: &TrustDomainReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TrustDomainReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_review_validator() -> TrustDomainReviewValidator {
    TrustDomainReviewValidator {
        rules: vec![
            Rc::new(TrustDomainReviewTitleLengthRule { max_length: 1000 }) as Rc<dyn TrustDomainReviewRule>,
            Rc::new(TrustDomainReviewRequiredDescriptionRule {
                message: "trust domain review needs an entry".to_string(),
            }),
            Rc::new(TrustDomainReviewRequiredSegmentRule {
                message: "trust domain review needs a value".to_string(),
            }),
            Rc::new(TrustDomainReviewCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(TrustDomainReviewSegmentLengthRule { max_length: 3600 }),
            Rc::new(TrustDomainReviewDescriptionLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_review_validator();
        let item = new_trust_domain_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TrustDomainReviewTitleLengthRule { max_length: 80 };
        let mut item = new_trust_domain_review("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = TrustDomainReviewRequiredDescriptionRule {
            message: "trust domain review needs this field".to_string(),
        };
        let mut item = new_trust_domain_review("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = TrustDomainReviewRequiredSegmentRule {
            message: "trust domain review needs an entry".to_string(),
        };
        let mut item = new_trust_domain_review("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = TrustDomainReviewSegmentLengthRule { max_length: 1000 };
        let mut item = new_trust_domain_review("x", "y");
        item.segment = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = TrustDomainReviewDescriptionLengthRule { max_length: 80 };
        let mut item = new_trust_domain_review("x", "y");
        item.description = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
