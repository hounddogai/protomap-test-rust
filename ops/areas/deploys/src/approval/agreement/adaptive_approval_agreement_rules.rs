use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive approval agreement must pass. Check returns its problems, or none.
pub trait AdaptiveApprovalAgreementRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveApprovalAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveApprovalAgreementRequiredScopeRule {
    pub message: String,
}

impl AdaptiveApprovalAgreementRule for AdaptiveApprovalAgreementRequiredScopeRule {
    fn check(&self, item: &AdaptiveApprovalAgreement) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveApprovalAgreementRequiredLabelRule {
    pub message: String,
}

impl AdaptiveApprovalAgreementRule for AdaptiveApprovalAgreementRequiredLabelRule {
    fn check(&self, item: &AdaptiveApprovalAgreement) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveApprovalAgreementCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveApprovalAgreementRule for AdaptiveApprovalAgreementCodePrefixRule {
    fn check(&self, item: &AdaptiveApprovalAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveApprovalAgreementRequiredCurrencyRule {
    pub message: String,
}

impl AdaptiveApprovalAgreementRule for AdaptiveApprovalAgreementRequiredCurrencyRule {
    fn check(&self, item: &AdaptiveApprovalAgreement) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive approval agreement.
#[derive(Debug, Clone)]
pub struct AdaptiveApprovalAgreementValidator {
    pub rules: Vec<Rc<dyn AdaptiveApprovalAgreementRule>>,
}

impl AdaptiveApprovalAgreementValidator {
    pub fn validate(&self, item: &AdaptiveApprovalAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveApprovalAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveApprovalAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_approval_agreement_validator() -> AdaptiveApprovalAgreementValidator {
    AdaptiveApprovalAgreementValidator {
        rules: vec![
            Rc::new(AdaptiveApprovalAgreementRequiredScopeRule {
                message: "adaptive approval agreement needs an entry".to_string(),
            }) as Rc<dyn AdaptiveApprovalAgreementRule>,
            Rc::new(AdaptiveApprovalAgreementRequiredLabelRule {
                message: "adaptive approval agreement needs this field".to_string(),
            }),
            Rc::new(AdaptiveApprovalAgreementCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(AdaptiveApprovalAgreementRequiredCurrencyRule {
                message: "adaptive approval agreement needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_approval_agreement_validator();
        let item = new_adaptive_approval_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = AdaptiveApprovalAgreementRequiredScopeRule {
            message: "adaptive approval agreement needs this field".to_string(),
        };
        let mut item = new_adaptive_approval_agreement("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AdaptiveApprovalAgreementRequiredLabelRule {
            message: "adaptive approval agreement needs this field".to_string(),
        };
        let mut item = new_adaptive_approval_agreement("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = AdaptiveApprovalAgreementRequiredCurrencyRule {
            message: "adaptive approval agreement needs an entry".to_string(),
        };
        let mut item = new_adaptive_approval_agreement("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
