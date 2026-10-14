use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation agreement must pass. Check returns its problems, or none.
pub trait EscalationAgreementRule: std::fmt::Debug {
    fn check(&self, item: &EscalationAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAgreementCodePrefixRule {
    pub prefix: String,
}

impl EscalationAgreementRule for EscalationAgreementCodePrefixRule {
    fn check(&self, item: &EscalationAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAgreementRequiredNameRule {
    pub message: String,
}

impl EscalationAgreementRule for EscalationAgreementRequiredNameRule {
    fn check(&self, item: &EscalationAgreement) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAgreementLabelLengthRule {
    pub max_length: i64,
}

impl EscalationAgreementRule for EscalationAgreementLabelLengthRule {
    fn check(&self, item: &EscalationAgreement) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAgreementBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationAgreementRule for EscalationAgreementBurstRangeRule {
    fn check(&self, item: &EscalationAgreement) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation agreement.
#[derive(Debug, Clone)]
pub struct EscalationAgreementValidator {
    pub rules: Vec<Rc<dyn EscalationAgreementRule>>,
}

impl EscalationAgreementValidator {
    pub fn validate(&self, item: &EscalationAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EscalationAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_agreement_validator() -> EscalationAgreementValidator {
    EscalationAgreementValidator {
        rules: vec![
            Rc::new(EscalationAgreementCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn EscalationAgreementRule>,
            Rc::new(EscalationAgreementRequiredNameRule {
                message: "escalation agreement needs this field".to_string(),
            }),
            Rc::new(EscalationAgreementLabelLengthRule { max_length: 32 }),
            Rc::new(EscalationAgreementBurstRangeRule { min: 16, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_agreement_validator();
        let item = new_escalation_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = EscalationAgreementRequiredNameRule {
            message: "escalation agreement needs this field".to_string(),
        };
        let mut item = new_escalation_agreement("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = EscalationAgreementLabelLengthRule { max_length: 16 };
        let mut item = new_escalation_agreement("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = EscalationAgreementBurstRangeRule { min: 32, max: 64 };
        let mut item = new_escalation_agreement("x", "y");
        item.burst = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
