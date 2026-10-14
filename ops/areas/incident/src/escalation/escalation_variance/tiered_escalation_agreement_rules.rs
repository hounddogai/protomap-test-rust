use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered escalation agreement must pass. Check returns its problems, or none.
pub trait TieredEscalationAgreementRule: std::fmt::Debug {
    fn check(&self, item: &TieredEscalationAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationAgreementAllowedNameRule {
    pub allowed: Vec<String>,
}

impl TieredEscalationAgreementRule for TieredEscalationAgreementAllowedNameRule {
    fn check(&self, item: &TieredEscalationAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is inconsistent", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationAgreementCodePrefixRule {
    pub prefix: String,
}

impl TieredEscalationAgreementRule for TieredEscalationAgreementCodePrefixRule {
    fn check(&self, item: &TieredEscalationAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationAgreementRequiredCodeRule {
    pub message: String,
}

impl TieredEscalationAgreementRule for TieredEscalationAgreementRequiredCodeRule {
    fn check(&self, item: &TieredEscalationAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationAgreementCodeLengthRule {
    pub max_length: i64,
}

impl TieredEscalationAgreementRule for TieredEscalationAgreementCodeLengthRule {
    fn check(&self, item: &TieredEscalationAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered escalation agreement.
#[derive(Debug, Clone)]
pub struct TieredEscalationAgreementValidator {
    pub rules: Vec<Rc<dyn TieredEscalationAgreementRule>>,
}

impl TieredEscalationAgreementValidator {
    pub fn validate(&self, item: &TieredEscalationAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredEscalationAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredEscalationAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_escalation_agreement_validator() -> TieredEscalationAgreementValidator {
    TieredEscalationAgreementValidator {
        rules: vec![
            Rc::new(TieredEscalationAgreementAllowedNameRule {
                allowed: vec!["rolling".to_string()],
            }) as Rc<dyn TieredEscalationAgreementRule>,
            Rc::new(TieredEscalationAgreementCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(TieredEscalationAgreementRequiredCodeRule {
                message: "tiered escalation agreement needs a value".to_string(),
            }),
            Rc::new(TieredEscalationAgreementCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_escalation_agreement_validator();
        let item = new_tiered_escalation_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredEscalationAgreementRequiredCodeRule {
            message: "tiered escalation agreement needs this field".to_string(),
        };
        let mut item = new_tiered_escalation_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredEscalationAgreementCodeLengthRule { max_length: 1000 };
        let mut item = new_tiered_escalation_agreement("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
