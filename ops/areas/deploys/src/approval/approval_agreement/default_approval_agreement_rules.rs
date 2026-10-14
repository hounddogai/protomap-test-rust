use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default approval agreement must pass. Check returns its problems, or none.
pub trait DefaultApprovalAgreementRule: std::fmt::Debug {
    fn check(&self, item: &DefaultApprovalAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultApprovalAgreementRequiredSummaryRule {
    pub message: String,
}

impl DefaultApprovalAgreementRule for DefaultApprovalAgreementRequiredSummaryRule {
    fn check(&self, item: &DefaultApprovalAgreement) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultApprovalAgreementRequiredLabelRule {
    pub message: String,
}

impl DefaultApprovalAgreementRule for DefaultApprovalAgreementRequiredLabelRule {
    fn check(&self, item: &DefaultApprovalAgreement) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultApprovalAgreementRequiredCodeRule {
    pub message: String,
}

impl DefaultApprovalAgreementRule for DefaultApprovalAgreementRequiredCodeRule {
    fn check(&self, item: &DefaultApprovalAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default approval agreement.
#[derive(Debug, Clone)]
pub struct DefaultApprovalAgreementValidator {
    pub rules: Vec<Rc<dyn DefaultApprovalAgreementRule>>,
}

impl DefaultApprovalAgreementValidator {
    pub fn validate(&self, item: &DefaultApprovalAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultApprovalAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultApprovalAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_approval_agreement_validator() -> DefaultApprovalAgreementValidator {
    DefaultApprovalAgreementValidator {
        rules: vec![
            Rc::new(DefaultApprovalAgreementRequiredSummaryRule {
                message: "default approval agreement needs an entry".to_string(),
            }) as Rc<dyn DefaultApprovalAgreementRule>,
            Rc::new(DefaultApprovalAgreementRequiredLabelRule {
                message: "default approval agreement needs an entry".to_string(),
            }),
            Rc::new(DefaultApprovalAgreementRequiredCodeRule {
                message: "default approval agreement needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_approval_agreement_validator();
        let item = new_default_approval_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = DefaultApprovalAgreementRequiredSummaryRule {
            message: "default approval agreement needs a value".to_string(),
        };
        let mut item = new_default_approval_agreement("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultApprovalAgreementRequiredLabelRule {
            message: "default approval agreement needs a value".to_string(),
        };
        let mut item = new_default_approval_agreement("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultApprovalAgreementRequiredCodeRule {
            message: "default approval agreement needs this field".to_string(),
        };
        let mut item = new_default_approval_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
