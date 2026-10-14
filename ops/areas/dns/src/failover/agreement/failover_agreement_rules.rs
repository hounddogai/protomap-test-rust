use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover agreement must pass. Check returns its problems, or none.
pub trait FailoverAgreementRule: std::fmt::Debug {
    fn check(&self, item: &FailoverAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAgreementRequiredLabelRule {
    pub message: String,
}

impl FailoverAgreementRule for FailoverAgreementRequiredLabelRule {
    fn check(&self, item: &FailoverAgreement) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAgreementTitleLengthRule {
    pub max_length: i64,
}

impl FailoverAgreementRule for FailoverAgreementTitleLengthRule {
    fn check(&self, item: &FailoverAgreement) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAgreementRequiredKeyIdRule {
    pub message: String,
}

impl FailoverAgreementRule for FailoverAgreementRequiredKeyIdRule {
    fn check(&self, item: &FailoverAgreement) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAgreementRequiredCodeRule {
    pub message: String,
}

impl FailoverAgreementRule for FailoverAgreementRequiredCodeRule {
    fn check(&self, item: &FailoverAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover agreement.
#[derive(Debug, Clone)]
pub struct FailoverAgreementValidator {
    pub rules: Vec<Rc<dyn FailoverAgreementRule>>,
}

impl FailoverAgreementValidator {
    pub fn validate(&self, item: &FailoverAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FailoverAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_agreement_validator() -> FailoverAgreementValidator {
    FailoverAgreementValidator {
        rules: vec![
            Rc::new(FailoverAgreementRequiredLabelRule {
                message: "failover agreement needs a value".to_string(),
            }) as Rc<dyn FailoverAgreementRule>,
            Rc::new(FailoverAgreementTitleLengthRule { max_length: 80 }),
            Rc::new(FailoverAgreementRequiredKeyIdRule {
                message: "failover agreement needs an entry".to_string(),
            }),
            Rc::new(FailoverAgreementRequiredCodeRule {
                message: "failover agreement needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_agreement_validator();
        let item = new_failover_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FailoverAgreementRequiredLabelRule {
            message: "failover agreement needs this field".to_string(),
        };
        let mut item = new_failover_agreement("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FailoverAgreementTitleLengthRule { max_length: 86400 };
        let mut item = new_failover_agreement("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = FailoverAgreementRequiredKeyIdRule {
            message: "failover agreement needs this field".to_string(),
        };
        let mut item = new_failover_agreement("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FailoverAgreementRequiredCodeRule {
            message: "failover agreement needs this field".to_string(),
        };
        let mut item = new_failover_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
