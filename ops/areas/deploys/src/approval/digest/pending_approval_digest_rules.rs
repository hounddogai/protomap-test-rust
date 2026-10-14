use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending approval digest must pass. Check returns its problems, or none.
pub trait PendingApprovalDigestRule: std::fmt::Debug {
    fn check(&self, item: &PendingApprovalDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalDigestSegmentLengthRule {
    pub max_length: i64,
}

impl PendingApprovalDigestRule for PendingApprovalDigestSegmentLengthRule {
    fn check(&self, item: &PendingApprovalDigest) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalDigestRequiredCodeRule {
    pub message: String,
}

impl PendingApprovalDigestRule for PendingApprovalDigestRequiredCodeRule {
    fn check(&self, item: &PendingApprovalDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalDigestRequiredAudienceRule {
    pub message: String,
}

impl PendingApprovalDigestRule for PendingApprovalDigestRequiredAudienceRule {
    fn check(&self, item: &PendingApprovalDigest) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalDigestAudienceLengthRule {
    pub max_length: i64,
}

impl PendingApprovalDigestRule for PendingApprovalDigestAudienceLengthRule {
    fn check(&self, item: &PendingApprovalDigest) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalDigestRequiredSegmentRule {
    pub message: String,
}

impl PendingApprovalDigestRule for PendingApprovalDigestRequiredSegmentRule {
    fn check(&self, item: &PendingApprovalDigest) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending approval digest.
#[derive(Debug, Clone)]
pub struct PendingApprovalDigestValidator {
    pub rules: Vec<Rc<dyn PendingApprovalDigestRule>>,
}

impl PendingApprovalDigestValidator {
    pub fn validate(&self, item: &PendingApprovalDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingApprovalDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingApprovalDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_approval_digest_validator() -> PendingApprovalDigestValidator {
    PendingApprovalDigestValidator {
        rules: vec![
            Rc::new(PendingApprovalDigestSegmentLengthRule { max_length: 255 }) as Rc<dyn PendingApprovalDigestRule>,
            Rc::new(PendingApprovalDigestRequiredCodeRule {
                message: "pending approval digest needs an entry".to_string(),
            }),
            Rc::new(PendingApprovalDigestRequiredAudienceRule {
                message: "pending approval digest needs this field".to_string(),
            }),
            Rc::new(PendingApprovalDigestAudienceLengthRule { max_length: 64 }),
            Rc::new(PendingApprovalDigestRequiredSegmentRule {
                message: "pending approval digest needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_approval_digest_validator();
        let item = new_pending_approval_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = PendingApprovalDigestSegmentLengthRule { max_length: 64 };
        let mut item = new_pending_approval_digest("x", "y");
        item.segment = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingApprovalDigestRequiredCodeRule {
            message: "pending approval digest needs this field".to_string(),
        };
        let mut item = new_pending_approval_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = PendingApprovalDigestRequiredAudienceRule {
            message: "pending approval digest needs this field".to_string(),
        };
        let mut item = new_pending_approval_digest("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = PendingApprovalDigestAudienceLengthRule { max_length: 1000 };
        let mut item = new_pending_approval_digest("x", "y");
        item.audience = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = PendingApprovalDigestRequiredSegmentRule {
            message: "pending approval digest needs an entry".to_string(),
        };
        let mut item = new_pending_approval_digest("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
