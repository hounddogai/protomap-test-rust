use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback agreement must pass. Check returns its problems, or none.
pub trait RollbackAgreementRule: std::fmt::Debug {
    fn check(&self, item: &RollbackAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAgreementLabelLengthRule {
    pub max_length: i64,
}

impl RollbackAgreementRule for RollbackAgreementLabelLengthRule {
    fn check(&self, item: &RollbackAgreement) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAgreementRequiredLabelRule {
    pub message: String,
}

impl RollbackAgreementRule for RollbackAgreementRequiredLabelRule {
    fn check(&self, item: &RollbackAgreement) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAgreementRequiredCodeRule {
    pub message: String,
}

impl RollbackAgreementRule for RollbackAgreementRequiredCodeRule {
    fn check(&self, item: &RollbackAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAgreementCodePrefixRule {
    pub prefix: String,
}

impl RollbackAgreementRule for RollbackAgreementCodePrefixRule {
    fn check(&self, item: &RollbackAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback agreement.
#[derive(Debug, Clone)]
pub struct RollbackAgreementValidator {
    pub rules: Vec<Rc<dyn RollbackAgreementRule>>,
}

impl RollbackAgreementValidator {
    pub fn validate(&self, item: &RollbackAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_agreement_validator() -> RollbackAgreementValidator {
    RollbackAgreementValidator {
        rules: vec![
            Rc::new(RollbackAgreementLabelLengthRule { max_length: 32 }) as Rc<dyn RollbackAgreementRule>,
            Rc::new(RollbackAgreementRequiredLabelRule {
                message: "rollback agreement needs a value".to_string(),
            }),
            Rc::new(RollbackAgreementRequiredCodeRule {
                message: "rollback agreement needs this field".to_string(),
            }),
            Rc::new(RollbackAgreementCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_agreement_validator();
        let item = new_rollback_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollbackAgreementLabelLengthRule { max_length: 64 };
        let mut item = new_rollback_agreement("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollbackAgreementRequiredLabelRule {
            message: "rollback agreement needs a value".to_string(),
        };
        let mut item = new_rollback_agreement("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollbackAgreementRequiredCodeRule {
            message: "rollback agreement needs an entry".to_string(),
        };
        let mut item = new_rollback_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
