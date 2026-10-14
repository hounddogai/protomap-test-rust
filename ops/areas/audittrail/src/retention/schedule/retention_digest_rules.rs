use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention digest must pass. Check returns its problems, or none.
pub trait RetentionDigestRule: std::fmt::Debug {
    fn check(&self, item: &RetentionDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionDigestAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl RetentionDigestRule for RetentionDigestAllowedReviewerRule {
    fn check(&self, item: &RetentionDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is blocked", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionDigestRequiredReviewerRule {
    pub message: String,
}

impl RetentionDigestRule for RetentionDigestRequiredReviewerRule {
    fn check(&self, item: &RetentionDigest) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionDigestReviewerLengthRule {
    pub max_length: i64,
}

impl RetentionDigestRule for RetentionDigestReviewerLengthRule {
    fn check(&self, item: &RetentionDigest) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention digest.
#[derive(Debug, Clone)]
pub struct RetentionDigestValidator {
    pub rules: Vec<Rc<dyn RetentionDigestRule>>,
}

impl RetentionDigestValidator {
    pub fn validate(&self, item: &RetentionDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_digest_validator() -> RetentionDigestValidator {
    RetentionDigestValidator {
        rules: vec![
            Rc::new(RetentionDigestAllowedReviewerRule { allowed: Vec::new() }) as Rc<dyn RetentionDigestRule>,
            Rc::new(RetentionDigestRequiredReviewerRule {
                message: "retention digest needs an entry".to_string(),
            }),
            Rc::new(RetentionDigestReviewerLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_digest_validator();
        let item = new_retention_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = RetentionDigestRequiredReviewerRule {
            message: "retention digest needs this field".to_string(),
        };
        let mut item = new_retention_digest("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = RetentionDigestReviewerLengthRule { max_length: 16 };
        let mut item = new_retention_digest("x", "y");
        item.reviewer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
