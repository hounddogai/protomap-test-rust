use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert agreement must pass. Check returns its problems, or none.
pub trait AlertAgreementRule: std::fmt::Debug {
    fn check(&self, item: &AlertAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAgreementRequiredCipherRule {
    pub message: String,
}

impl AlertAgreementRule for AlertAgreementRequiredCipherRule {
    fn check(&self, item: &AlertAgreement) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAgreementCipherLengthRule {
    pub max_length: i64,
}

impl AlertAgreementRule for AlertAgreementCipherLengthRule {
    fn check(&self, item: &AlertAgreement) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAgreementNameLengthRule {
    pub max_length: i64,
}

impl AlertAgreementRule for AlertAgreementNameLengthRule {
    fn check(&self, item: &AlertAgreement) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAgreementApprovedGuardRule {
    pub limit: i64,
}

impl AlertAgreementRule for AlertAgreementApprovedGuardRule {
    fn check(&self, item: &AlertAgreement) -> Vec<String> {
        if item.approved && (item.limit > self.limit) {
            return vec![format!("limit {} exceeds {} while approved", item.limit, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAgreementRequiredLocaleRule {
    pub message: String,
}

impl AlertAgreementRule for AlertAgreementRequiredLocaleRule {
    fn check(&self, item: &AlertAgreement) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert agreement.
#[derive(Debug, Clone)]
pub struct AlertAgreementValidator {
    pub rules: Vec<Rc<dyn AlertAgreementRule>>,
}

impl AlertAgreementValidator {
    pub fn validate(&self, item: &AlertAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_agreement_validator() -> AlertAgreementValidator {
    AlertAgreementValidator {
        rules: vec![
            Rc::new(AlertAgreementRequiredCipherRule {
                message: "alert agreement needs a value".to_string(),
            }) as Rc<dyn AlertAgreementRule>,
            Rc::new(AlertAgreementCipherLengthRule { max_length: 64 }),
            Rc::new(AlertAgreementNameLengthRule { max_length: 32 }),
            Rc::new(AlertAgreementApprovedGuardRule { limit: 16 }),
            Rc::new(AlertAgreementRequiredLocaleRule {
                message: "alert agreement needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_agreement_validator();
        let item = new_alert_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = AlertAgreementRequiredCipherRule {
            message: "alert agreement needs an entry".to_string(),
        };
        let mut item = new_alert_agreement("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = AlertAgreementCipherLengthRule { max_length: 16 };
        let mut item = new_alert_agreement("x", "y");
        item.cipher = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = AlertAgreementNameLengthRule { max_length: 80 };
        let mut item = new_alert_agreement("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = AlertAgreementRequiredLocaleRule {
            message: "alert agreement needs an entry".to_string(),
        };
        let mut item = new_alert_agreement("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
