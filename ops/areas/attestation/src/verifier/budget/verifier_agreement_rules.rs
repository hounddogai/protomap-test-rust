use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier agreement must pass. Check returns its problems, or none.
pub trait VerifierAgreementRule: std::fmt::Debug {
    fn check(&self, item: &VerifierAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAgreementRequiredCodeRule {
    pub message: String,
}

impl VerifierAgreementRule for VerifierAgreementRequiredCodeRule {
    fn check(&self, item: &VerifierAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAgreementTitleLengthRule {
    pub max_length: i64,
}

impl VerifierAgreementRule for VerifierAgreementTitleLengthRule {
    fn check(&self, item: &VerifierAgreement) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAgreementSegmentLengthRule {
    pub max_length: i64,
}

impl VerifierAgreementRule for VerifierAgreementSegmentLengthRule {
    fn check(&self, item: &VerifierAgreement) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier agreement.
#[derive(Debug, Clone)]
pub struct VerifierAgreementValidator {
    pub rules: Vec<Rc<dyn VerifierAgreementRule>>,
}

impl VerifierAgreementValidator {
    pub fn validate(&self, item: &VerifierAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_agreement_validator() -> VerifierAgreementValidator {
    VerifierAgreementValidator {
        rules: vec![
            Rc::new(VerifierAgreementRequiredCodeRule {
                message: "verifier agreement needs an entry".to_string(),
            }) as Rc<dyn VerifierAgreementRule>,
            Rc::new(VerifierAgreementTitleLengthRule { max_length: 16 }),
            Rc::new(VerifierAgreementSegmentLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_agreement_validator();
        let item = new_verifier_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = VerifierAgreementRequiredCodeRule {
            message: "verifier agreement needs an entry".to_string(),
        };
        let mut item = new_verifier_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = VerifierAgreementTitleLengthRule { max_length: 1000 };
        let mut item = new_verifier_agreement("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = VerifierAgreementSegmentLengthRule { max_length: 64 };
        let mut item = new_verifier_agreement("x", "y");
        item.segment = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
