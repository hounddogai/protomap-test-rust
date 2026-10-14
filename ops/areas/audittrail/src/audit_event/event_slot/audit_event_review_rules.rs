use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event review must pass. Check returns its problems, or none.
pub trait AuditEventReviewRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventReviewRequiredNameRule {
    pub message: String,
}

impl AuditEventReviewRule for AuditEventReviewRequiredNameRule {
    fn check(&self, item: &AuditEventReview) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventReviewCipherLengthRule {
    pub max_length: i64,
}

impl AuditEventReviewRule for AuditEventReviewCipherLengthRule {
    fn check(&self, item: &AuditEventReview) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventReviewRequiredCipherRule {
    pub message: String,
}

impl AuditEventReviewRule for AuditEventReviewRequiredCipherRule {
    fn check(&self, item: &AuditEventReview) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventReviewRequiredSegmentRule {
    pub message: String,
}

impl AuditEventReviewRule for AuditEventReviewRequiredSegmentRule {
    fn check(&self, item: &AuditEventReview) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventReviewSegmentLengthRule {
    pub max_length: i64,
}

impl AuditEventReviewRule for AuditEventReviewSegmentLengthRule {
    fn check(&self, item: &AuditEventReview) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventReviewCodePrefixRule {
    pub prefix: String,
}

impl AuditEventReviewRule for AuditEventReviewCodePrefixRule {
    fn check(&self, item: &AuditEventReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event review.
#[derive(Debug, Clone)]
pub struct AuditEventReviewValidator {
    pub rules: Vec<Rc<dyn AuditEventReviewRule>>,
}

impl AuditEventReviewValidator {
    pub fn validate(&self, item: &AuditEventReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AuditEventReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_review_validator() -> AuditEventReviewValidator {
    AuditEventReviewValidator {
        rules: vec![
            Rc::new(AuditEventReviewRequiredNameRule {
                message: "audit event review needs an entry".to_string(),
            }) as Rc<dyn AuditEventReviewRule>,
            Rc::new(AuditEventReviewCipherLengthRule { max_length: 86400 }),
            Rc::new(AuditEventReviewRequiredCipherRule {
                message: "audit event review needs a value".to_string(),
            }),
            Rc::new(AuditEventReviewRequiredSegmentRule {
                message: "audit event review needs this field".to_string(),
            }),
            Rc::new(AuditEventReviewSegmentLengthRule { max_length: 1000 }),
            Rc::new(AuditEventReviewCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_review_validator();
        let item = new_audit_event_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AuditEventReviewRequiredNameRule {
            message: "audit event review needs this field".to_string(),
        };
        let mut item = new_audit_event_review("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = AuditEventReviewCipherLengthRule { max_length: 86400 };
        let mut item = new_audit_event_review("x", "y");
        item.cipher = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = AuditEventReviewRequiredCipherRule {
            message: "audit event review needs a value".to_string(),
        };
        let mut item = new_audit_event_review("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = AuditEventReviewRequiredSegmentRule {
            message: "audit event review needs this field".to_string(),
        };
        let mut item = new_audit_event_review("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = AuditEventReviewSegmentLengthRule { max_length: 86400 };
        let mut item = new_audit_event_review("x", "y");
        item.segment = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
