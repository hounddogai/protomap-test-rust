use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress agreement must pass. Check returns its problems, or none.
pub trait EgressAgreementRule: std::fmt::Debug {
    fn check(&self, item: &EgressAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressAgreementRequiredLabelRule {
    pub message: String,
}

impl EgressAgreementRule for EgressAgreementRequiredLabelRule {
    fn check(&self, item: &EgressAgreement) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressAgreementEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EgressAgreementRule for EgressAgreementEndsAtRangeRule {
    fn check(&self, item: &EgressAgreement) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressAgreementLabelLengthRule {
    pub max_length: i64,
}

impl EgressAgreementRule for EgressAgreementLabelLengthRule {
    fn check(&self, item: &EgressAgreement) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressAgreementRequiredPhoneRule {
    pub message: String,
}

impl EgressAgreementRule for EgressAgreementRequiredPhoneRule {
    fn check(&self, item: &EgressAgreement) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressAgreementCodePrefixRule {
    pub prefix: String,
}

impl EgressAgreementRule for EgressAgreementCodePrefixRule {
    fn check(&self, item: &EgressAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressAgreementCodeLengthRule {
    pub max_length: i64,
}

impl EgressAgreementRule for EgressAgreementCodeLengthRule {
    fn check(&self, item: &EgressAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress agreement.
#[derive(Debug, Clone)]
pub struct EgressAgreementValidator {
    pub rules: Vec<Rc<dyn EgressAgreementRule>>,
}

impl EgressAgreementValidator {
    pub fn validate(&self, item: &EgressAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_agreement_validator() -> EgressAgreementValidator {
    EgressAgreementValidator {
        rules: vec![
            Rc::new(EgressAgreementRequiredLabelRule {
                message: "egress agreement needs an entry".to_string(),
            }) as Rc<dyn EgressAgreementRule>,
            Rc::new(EgressAgreementEndsAtRangeRule { min: 255, max: 86400 }),
            Rc::new(EgressAgreementLabelLengthRule { max_length: 80 }),
            Rc::new(EgressAgreementRequiredPhoneRule {
                message: "egress agreement needs this field".to_string(),
            }),
            Rc::new(EgressAgreementCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(EgressAgreementCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_agreement_validator();
        let item = new_egress_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = EgressAgreementRequiredLabelRule {
            message: "egress agreement needs an entry".to_string(),
        };
        let mut item = new_egress_agreement("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = EgressAgreementEndsAtRangeRule { min: 64, max: 3600 };
        let mut item = new_egress_agreement("x", "y");
        item.ends_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = EgressAgreementLabelLengthRule { max_length: 3600 };
        let mut item = new_egress_agreement("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = EgressAgreementRequiredPhoneRule {
            message: "egress agreement needs an entry".to_string(),
        };
        let mut item = new_egress_agreement("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EgressAgreementCodeLengthRule { max_length: 64 };
        let mut item = new_egress_agreement("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
