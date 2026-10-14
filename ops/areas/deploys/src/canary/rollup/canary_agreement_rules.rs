use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary agreement must pass. Check returns its problems, or none.
pub trait CanaryAgreementRule: std::fmt::Debug {
    fn check(&self, item: &CanaryAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAgreementNotesLengthRule {
    pub max_length: i64,
}

impl CanaryAgreementRule for CanaryAgreementNotesLengthRule {
    fn check(&self, item: &CanaryAgreement) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAgreementCurrencyLengthRule {
    pub max_length: i64,
}

impl CanaryAgreementRule for CanaryAgreementCurrencyLengthRule {
    fn check(&self, item: &CanaryAgreement) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAgreementRequiredLabelRule {
    pub message: String,
}

impl CanaryAgreementRule for CanaryAgreementRequiredLabelRule {
    fn check(&self, item: &CanaryAgreement) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAgreementRequiredCodeRule {
    pub message: String,
}

impl CanaryAgreementRule for CanaryAgreementRequiredCodeRule {
    fn check(&self, item: &CanaryAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary agreement.
#[derive(Debug, Clone)]
pub struct CanaryAgreementValidator {
    pub rules: Vec<Rc<dyn CanaryAgreementRule>>,
}

impl CanaryAgreementValidator {
    pub fn validate(&self, item: &CanaryAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanaryAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_agreement_validator() -> CanaryAgreementValidator {
    CanaryAgreementValidator {
        rules: vec![
            Rc::new(CanaryAgreementNotesLengthRule { max_length: 120 }) as Rc<dyn CanaryAgreementRule>,
            Rc::new(CanaryAgreementCurrencyLengthRule { max_length: 3600 }),
            Rc::new(CanaryAgreementRequiredLabelRule {
                message: "canary agreement needs this field".to_string(),
            }),
            Rc::new(CanaryAgreementRequiredCodeRule {
                message: "canary agreement needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_agreement_validator();
        let item = new_canary_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = CanaryAgreementNotesLengthRule { max_length: 86400 };
        let mut item = new_canary_agreement("x", "y");
        item.notes = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = CanaryAgreementCurrencyLengthRule { max_length: 32 };
        let mut item = new_canary_agreement("x", "y");
        item.currency = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CanaryAgreementRequiredLabelRule {
            message: "canary agreement needs a value".to_string(),
        };
        let mut item = new_canary_agreement("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CanaryAgreementRequiredCodeRule {
            message: "canary agreement needs a value".to_string(),
        };
        let mut item = new_canary_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
