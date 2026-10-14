use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check agreement must pass. Check returns its problems, or none.
pub trait HealthCheckAgreementRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAgreementReviewerLengthRule {
    pub max_length: i64,
}

impl HealthCheckAgreementRule for HealthCheckAgreementReviewerLengthRule {
    fn check(&self, item: &HealthCheckAgreement) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAgreementCipherLengthRule {
    pub max_length: i64,
}

impl HealthCheckAgreementRule for HealthCheckAgreementCipherLengthRule {
    fn check(&self, item: &HealthCheckAgreement) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAgreementRequiredCodeRule {
    pub message: String,
}

impl HealthCheckAgreementRule for HealthCheckAgreementRequiredCodeRule {
    fn check(&self, item: &HealthCheckAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAgreementCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckAgreementRule for HealthCheckAgreementCeilingRangeRule {
    fn check(&self, item: &HealthCheckAgreement) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAgreementCodeLengthRule {
    pub max_length: i64,
}

impl HealthCheckAgreementRule for HealthCheckAgreementCodeLengthRule {
    fn check(&self, item: &HealthCheckAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAgreementRequiredReviewerRule {
    pub message: String,
}

impl HealthCheckAgreementRule for HealthCheckAgreementRequiredReviewerRule {
    fn check(&self, item: &HealthCheckAgreement) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check agreement.
#[derive(Debug, Clone)]
pub struct HealthCheckAgreementValidator {
    pub rules: Vec<Rc<dyn HealthCheckAgreementRule>>,
}

impl HealthCheckAgreementValidator {
    pub fn validate(&self, item: &HealthCheckAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_agreement_validator() -> HealthCheckAgreementValidator {
    HealthCheckAgreementValidator {
        rules: vec![
            Rc::new(HealthCheckAgreementReviewerLengthRule { max_length: 86400 }) as Rc<dyn HealthCheckAgreementRule>,
            Rc::new(HealthCheckAgreementCipherLengthRule { max_length: 80 }),
            Rc::new(HealthCheckAgreementRequiredCodeRule {
                message: "health check agreement needs a value".to_string(),
            }),
            Rc::new(HealthCheckAgreementCeilingRangeRule { min: 1.6, max: 4021.2 }),
            Rc::new(HealthCheckAgreementCodeLengthRule { max_length: 1000 }),
            Rc::new(HealthCheckAgreementRequiredReviewerRule {
                message: "health check agreement needs an entry".to_string(),
            }),
        ],
    }
}
