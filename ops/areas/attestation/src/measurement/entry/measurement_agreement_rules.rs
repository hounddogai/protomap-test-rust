use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement agreement must pass. Check returns its problems, or none.
pub trait MeasurementAgreementRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAgreementBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementAgreementRule for MeasurementAgreementBudgetRangeRule {
    fn check(&self, item: &MeasurementAgreement) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAgreementRequiredCodeRule {
    pub message: String,
}

impl MeasurementAgreementRule for MeasurementAgreementRequiredCodeRule {
    fn check(&self, item: &MeasurementAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAgreementCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementAgreementRule for MeasurementAgreementCeilingRangeRule {
    fn check(&self, item: &MeasurementAgreement) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAgreementUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementAgreementRule for MeasurementAgreementUpdatedAtRangeRule {
    fn check(&self, item: &MeasurementAgreement) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAgreementCodePrefixRule {
    pub prefix: String,
}

impl MeasurementAgreementRule for MeasurementAgreementCodePrefixRule {
    fn check(&self, item: &MeasurementAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAgreementCodeLengthRule {
    pub max_length: i64,
}

impl MeasurementAgreementRule for MeasurementAgreementCodeLengthRule {
    fn check(&self, item: &MeasurementAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement agreement.
#[derive(Debug, Clone)]
pub struct MeasurementAgreementValidator {
    pub rules: Vec<Rc<dyn MeasurementAgreementRule>>,
}

impl MeasurementAgreementValidator {
    pub fn validate(&self, item: &MeasurementAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_agreement_validator() -> MeasurementAgreementValidator {
    MeasurementAgreementValidator {
        rules: vec![
            Rc::new(MeasurementAgreementBudgetRangeRule { min: 3.2, max: 3785.3 }) as Rc<dyn MeasurementAgreementRule>,
            Rc::new(MeasurementAgreementRequiredCodeRule {
                message: "measurement agreement needs an entry".to_string(),
            }),
            Rc::new(MeasurementAgreementCeilingRangeRule { min: 8.8, max: 1019.3 }),
            Rc::new(MeasurementAgreementUpdatedAtRangeRule { min: 32, max: 64 }),
            Rc::new(MeasurementAgreementCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(MeasurementAgreementCodeLengthRule { max_length: 1000 }),
        ],
    }
}
