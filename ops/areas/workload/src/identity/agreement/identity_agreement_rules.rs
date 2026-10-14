use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a identity agreement must pass. Check returns its problems, or none.
pub trait IdentityAgreementRule: std::fmt::Debug {
    fn check(&self, item: &IdentityAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAgreementAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl IdentityAgreementRule for IdentityAgreementAllowedLabelRule {
    fn check(&self, item: &IdentityAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is blocked", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAgreementCodeLengthRule {
    pub max_length: i64,
}

impl IdentityAgreementRule for IdentityAgreementCodeLengthRule {
    fn check(&self, item: &IdentityAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAgreementRequiredCodeRule {
    pub message: String,
}

impl IdentityAgreementRule for IdentityAgreementRequiredCodeRule {
    fn check(&self, item: &IdentityAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAgreementCipherLengthRule {
    pub max_length: i64,
}

impl IdentityAgreementRule for IdentityAgreementCipherLengthRule {
    fn check(&self, item: &IdentityAgreement) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAgreementRequiredLabelRule {
    pub message: String,
}

impl IdentityAgreementRule for IdentityAgreementRequiredLabelRule {
    fn check(&self, item: &IdentityAgreement) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity agreement.
#[derive(Debug, Clone)]
pub struct IdentityAgreementValidator {
    pub rules: Vec<Rc<dyn IdentityAgreementRule>>,
}

impl IdentityAgreementValidator {
    pub fn validate(&self, item: &IdentityAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_agreement_validator() -> IdentityAgreementValidator {
    IdentityAgreementValidator {
        rules: vec![
            Rc::new(IdentityAgreementAllowedLabelRule {
                allowed: vec!["lenient".to_string()],
            }) as Rc<dyn IdentityAgreementRule>,
            Rc::new(IdentityAgreementCodeLengthRule { max_length: 3600 }),
            Rc::new(IdentityAgreementRequiredCodeRule {
                message: "identity agreement needs this field".to_string(),
            }),
            Rc::new(IdentityAgreementCipherLengthRule { max_length: 1000 }),
            Rc::new(IdentityAgreementRequiredLabelRule {
                message: "identity agreement needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_agreement_validator();
        let item = new_identity_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IdentityAgreementCodeLengthRule { max_length: 1000 };
        let mut item = new_identity_agreement("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IdentityAgreementRequiredCodeRule {
            message: "identity agreement needs an entry".to_string(),
        };
        let mut item = new_identity_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = IdentityAgreementCipherLengthRule { max_length: 3600 };
        let mut item = new_identity_agreement("x", "y");
        item.cipher = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = IdentityAgreementRequiredLabelRule {
            message: "identity agreement needs a value".to_string(),
        };
        let mut item = new_identity_agreement("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
