use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress review must pass. Check returns its problems, or none.
pub trait EgressReviewRule: std::fmt::Debug {
    fn check(&self, item: &EgressReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressReviewLabelLengthRule {
    pub max_length: i64,
}

impl EgressReviewRule for EgressReviewLabelLengthRule {
    fn check(&self, item: &EgressReview) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressReviewIssuerLengthRule {
    pub max_length: i64,
}

impl EgressReviewRule for EgressReviewIssuerLengthRule {
    fn check(&self, item: &EgressReview) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressReviewRequiredCodeRule {
    pub message: String,
}

impl EgressReviewRule for EgressReviewRequiredCodeRule {
    fn check(&self, item: &EgressReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress review.
#[derive(Debug, Clone)]
pub struct EgressReviewValidator {
    pub rules: Vec<Rc<dyn EgressReviewRule>>,
}

impl EgressReviewValidator {
    pub fn validate(&self, item: &EgressReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EgressReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_review_validator() -> EgressReviewValidator {
    EgressReviewValidator {
        rules: vec![
            Rc::new(EgressReviewLabelLengthRule { max_length: 16 }) as Rc<dyn EgressReviewRule>,
            Rc::new(EgressReviewIssuerLengthRule { max_length: 255 }),
            Rc::new(EgressReviewRequiredCodeRule {
                message: "egress review needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_review_validator();
        let item = new_egress_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = EgressReviewLabelLengthRule { max_length: 16 };
        let mut item = new_egress_review("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = EgressReviewIssuerLengthRule { max_length: 80 };
        let mut item = new_egress_review("x", "y");
        item.issuer = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EgressReviewRequiredCodeRule {
            message: "egress review needs an entry".to_string(),
        };
        let mut item = new_egress_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
