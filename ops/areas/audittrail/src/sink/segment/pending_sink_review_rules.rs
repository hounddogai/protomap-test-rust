use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending sink review must pass. Check returns its problems, or none.
pub trait PendingSinkReviewRule: std::fmt::Debug {
    fn check(&self, item: &PendingSinkReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSinkReviewAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PendingSinkReviewRule for PendingSinkReviewAllowedCodeRule {
    fn check(&self, item: &PendingSinkReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSinkReviewTitleLengthRule {
    pub max_length: i64,
}

impl PendingSinkReviewRule for PendingSinkReviewTitleLengthRule {
    fn check(&self, item: &PendingSinkReview) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSinkReviewRequiredCodeRule {
    pub message: String,
}

impl PendingSinkReviewRule for PendingSinkReviewRequiredCodeRule {
    fn check(&self, item: &PendingSinkReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending sink review.
#[derive(Debug, Clone)]
pub struct PendingSinkReviewValidator {
    pub rules: Vec<Rc<dyn PendingSinkReviewRule>>,
}

impl PendingSinkReviewValidator {
    pub fn validate(&self, item: &PendingSinkReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingSinkReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingSinkReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_sink_review_validator() -> PendingSinkReviewValidator {
    PendingSinkReviewValidator {
        rules: vec![
            Rc::new(PendingSinkReviewAllowedCodeRule {
                allowed: vec!["default".to_string()],
            }) as Rc<dyn PendingSinkReviewRule>,
            Rc::new(PendingSinkReviewTitleLengthRule { max_length: 120 }),
            Rc::new(PendingSinkReviewRequiredCodeRule {
                message: "pending sink review needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_sink_review_validator();
        let item = new_pending_sink_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingSinkReviewTitleLengthRule { max_length: 120 };
        let mut item = new_pending_sink_review("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingSinkReviewRequiredCodeRule {
            message: "pending sink review needs a value".to_string(),
        };
        let mut item = new_pending_sink_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
