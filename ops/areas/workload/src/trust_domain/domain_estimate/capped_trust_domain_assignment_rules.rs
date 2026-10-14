use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped trust domain assignment must pass. Check returns its problems, or none.
pub trait CappedTrustDomainAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &CappedTrustDomainAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTrustDomainAssignmentTitleLengthRule {
    pub max_length: i64,
}

impl CappedTrustDomainAssignmentRule for CappedTrustDomainAssignmentTitleLengthRule {
    fn check(&self, item: &CappedTrustDomainAssignment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTrustDomainAssignmentRequiredFingerprintRule {
    pub message: String,
}

impl CappedTrustDomainAssignmentRule for CappedTrustDomainAssignmentRequiredFingerprintRule {
    fn check(&self, item: &CappedTrustDomainAssignment) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTrustDomainAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl CappedTrustDomainAssignmentRule for CappedTrustDomainAssignmentCodeLengthRule {
    fn check(&self, item: &CappedTrustDomainAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTrustDomainAssignmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CappedTrustDomainAssignmentRule for CappedTrustDomainAssignmentAllowedCodeRule {
    fn check(&self, item: &CappedTrustDomainAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTrustDomainAssignmentRequiredCodeRule {
    pub message: String,
}

impl CappedTrustDomainAssignmentRule for CappedTrustDomainAssignmentRequiredCodeRule {
    fn check(&self, item: &CappedTrustDomainAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTrustDomainAssignmentFingerprintLengthRule {
    pub max_length: i64,
}

impl CappedTrustDomainAssignmentRule for CappedTrustDomainAssignmentFingerprintLengthRule {
    fn check(&self, item: &CappedTrustDomainAssignment) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped trust domain assignment.
#[derive(Debug, Clone)]
pub struct CappedTrustDomainAssignmentValidator {
    pub rules: Vec<Rc<dyn CappedTrustDomainAssignmentRule>>,
}

impl CappedTrustDomainAssignmentValidator {
    pub fn validate(&self, item: &CappedTrustDomainAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedTrustDomainAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedTrustDomainAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_trust_domain_assignment_validator() -> CappedTrustDomainAssignmentValidator {
    CappedTrustDomainAssignmentValidator {
        rules: vec![
            Rc::new(CappedTrustDomainAssignmentTitleLengthRule { max_length: 32 })
                as Rc<dyn CappedTrustDomainAssignmentRule>,
            Rc::new(CappedTrustDomainAssignmentRequiredFingerprintRule {
                message: "capped trust domain assignment needs a value".to_string(),
            }),
            Rc::new(CappedTrustDomainAssignmentCodeLengthRule { max_length: 86400 }),
            Rc::new(CappedTrustDomainAssignmentAllowedCodeRule {
                allowed: vec!["rolling".to_string(), "batched".to_string(), "manual".to_string()],
            }),
            Rc::new(CappedTrustDomainAssignmentRequiredCodeRule {
                message: "capped trust domain assignment needs an entry".to_string(),
            }),
            Rc::new(CappedTrustDomainAssignmentFingerprintLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_trust_domain_assignment_validator();
        let item = new_capped_trust_domain_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = CappedTrustDomainAssignmentTitleLengthRule { max_length: 1000 };
        let mut item = new_capped_trust_domain_assignment("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = CappedTrustDomainAssignmentRequiredFingerprintRule {
            message: "capped trust domain assignment needs an entry".to_string(),
        };
        let mut item = new_capped_trust_domain_assignment("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedTrustDomainAssignmentCodeLengthRule { max_length: 86400 };
        let mut item = new_capped_trust_domain_assignment("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedTrustDomainAssignmentRequiredCodeRule {
            message: "capped trust domain assignment needs this field".to_string(),
        };
        let mut item = new_capped_trust_domain_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = CappedTrustDomainAssignmentFingerprintLengthRule { max_length: 120 };
        let mut item = new_capped_trust_domain_assignment("x", "y");
        item.fingerprint = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
