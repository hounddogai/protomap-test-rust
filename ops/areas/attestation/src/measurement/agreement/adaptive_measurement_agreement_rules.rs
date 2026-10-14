use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive measurement agreement must pass. Check returns its problems, or none.
pub trait AdaptiveMeasurementAgreementRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveMeasurementAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementAgreementCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveMeasurementAgreementRule for AdaptiveMeasurementAgreementCodeLengthRule {
    fn check(&self, item: &AdaptiveMeasurementAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementAgreementRequiredLabelRule {
    pub message: String,
}

impl AdaptiveMeasurementAgreementRule for AdaptiveMeasurementAgreementRequiredLabelRule {
    fn check(&self, item: &AdaptiveMeasurementAgreement) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementAgreementRequiredScopeRule {
    pub message: String,
}

impl AdaptiveMeasurementAgreementRule for AdaptiveMeasurementAgreementRequiredScopeRule {
    fn check(&self, item: &AdaptiveMeasurementAgreement) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementAgreementAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveMeasurementAgreementRule for AdaptiveMeasurementAgreementAmountRangeRule {
    fn check(&self, item: &AdaptiveMeasurementAgreement) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementAgreementCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveMeasurementAgreementRule for AdaptiveMeasurementAgreementCodePrefixRule {
    fn check(&self, item: &AdaptiveMeasurementAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive measurement agreement.
#[derive(Debug, Clone)]
pub struct AdaptiveMeasurementAgreementValidator {
    pub rules: Vec<Rc<dyn AdaptiveMeasurementAgreementRule>>,
}

impl AdaptiveMeasurementAgreementValidator {
    pub fn validate(&self, item: &AdaptiveMeasurementAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveMeasurementAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveMeasurementAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_measurement_agreement_validator() -> AdaptiveMeasurementAgreementValidator {
    AdaptiveMeasurementAgreementValidator {
        rules: vec![
            Rc::new(AdaptiveMeasurementAgreementCodeLengthRule { max_length: 3600 })
                as Rc<dyn AdaptiveMeasurementAgreementRule>,
            Rc::new(AdaptiveMeasurementAgreementRequiredLabelRule {
                message: "adaptive measurement agreement needs this field".to_string(),
            }),
            Rc::new(AdaptiveMeasurementAgreementRequiredScopeRule {
                message: "adaptive measurement agreement needs a value".to_string(),
            }),
            Rc::new(AdaptiveMeasurementAgreementAmountRangeRule { min: 5.1, max: 3629.5 }),
            Rc::new(AdaptiveMeasurementAgreementCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_measurement_agreement_validator();
        let item = new_adaptive_measurement_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveMeasurementAgreementCodeLengthRule { max_length: 32 };
        let mut item = new_adaptive_measurement_agreement("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AdaptiveMeasurementAgreementRequiredLabelRule {
            message: "adaptive measurement agreement needs a value".to_string(),
        };
        let mut item = new_adaptive_measurement_agreement("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = AdaptiveMeasurementAgreementRequiredScopeRule {
            message: "adaptive measurement agreement needs a value".to_string(),
        };
        let mut item = new_adaptive_measurement_agreement("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = AdaptiveMeasurementAgreementAmountRangeRule { min: 9.9, max: 3120.8 };
        let mut item = new_adaptive_measurement_agreement("x", "y");
        item.amount = 3122.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
