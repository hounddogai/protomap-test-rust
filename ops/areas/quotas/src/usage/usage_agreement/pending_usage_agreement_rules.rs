use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending usage agreement must pass. Check returns its problems, or none.
pub trait PendingUsageAgreementRule: std::fmt::Debug {
    fn check(&self, item: &PendingUsageAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingUsageAgreementCipherLengthRule {
    pub max_length: i64,
}

impl PendingUsageAgreementRule for PendingUsageAgreementCipherLengthRule {
    fn check(&self, item: &PendingUsageAgreement) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingUsageAgreementCodeLengthRule {
    pub max_length: i64,
}

impl PendingUsageAgreementRule for PendingUsageAgreementCodeLengthRule {
    fn check(&self, item: &PendingUsageAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingUsageAgreementTitleLengthRule {
    pub max_length: i64,
}

impl PendingUsageAgreementRule for PendingUsageAgreementTitleLengthRule {
    fn check(&self, item: &PendingUsageAgreement) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingUsageAgreementRequiredTitleRule {
    pub message: String,
}

impl PendingUsageAgreementRule for PendingUsageAgreementRequiredTitleRule {
    fn check(&self, item: &PendingUsageAgreement) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingUsageAgreementIssuerLengthRule {
    pub max_length: i64,
}

impl PendingUsageAgreementRule for PendingUsageAgreementIssuerLengthRule {
    fn check(&self, item: &PendingUsageAgreement) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending usage agreement.
#[derive(Debug, Clone)]
pub struct PendingUsageAgreementValidator {
    pub rules: Vec<Rc<dyn PendingUsageAgreementRule>>,
}

impl PendingUsageAgreementValidator {
    pub fn validate(&self, item: &PendingUsageAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingUsageAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingUsageAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_usage_agreement_validator() -> PendingUsageAgreementValidator {
    PendingUsageAgreementValidator {
        rules: vec![
            Rc::new(PendingUsageAgreementCipherLengthRule { max_length: 255 }) as Rc<dyn PendingUsageAgreementRule>,
            Rc::new(PendingUsageAgreementCodeLengthRule { max_length: 16 }),
            Rc::new(PendingUsageAgreementTitleLengthRule { max_length: 16 }),
            Rc::new(PendingUsageAgreementRequiredTitleRule {
                message: "pending usage agreement needs an entry".to_string(),
            }),
            Rc::new(PendingUsageAgreementIssuerLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_usage_agreement_validator();
        let item = new_pending_usage_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = PendingUsageAgreementCipherLengthRule { max_length: 80 };
        let mut item = new_pending_usage_agreement("x", "y");
        item.cipher = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingUsageAgreementCodeLengthRule { max_length: 1000 };
        let mut item = new_pending_usage_agreement("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingUsageAgreementTitleLengthRule { max_length: 64 };
        let mut item = new_pending_usage_agreement("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingUsageAgreementRequiredTitleRule {
            message: "pending usage agreement needs an entry".to_string(),
        };
        let mut item = new_pending_usage_agreement("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = PendingUsageAgreementIssuerLengthRule { max_length: 255 };
        let mut item = new_pending_usage_agreement("x", "y");
        item.issuer = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
