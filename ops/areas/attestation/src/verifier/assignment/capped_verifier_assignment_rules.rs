use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped verifier assignment must pass. Check returns its problems, or none.
pub trait CappedVerifierAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &CappedVerifierAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierAssignmentThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedVerifierAssignmentRule for CappedVerifierAssignmentThresholdRangeRule {
    fn check(&self, item: &CappedVerifierAssignment) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierAssignmentCurrencyLengthRule {
    pub max_length: i64,
}

impl CappedVerifierAssignmentRule for CappedVerifierAssignmentCurrencyLengthRule {
    fn check(&self, item: &CappedVerifierAssignment) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl CappedVerifierAssignmentRule for CappedVerifierAssignmentCodeLengthRule {
    fn check(&self, item: &CappedVerifierAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierAssignmentAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl CappedVerifierAssignmentRule for CappedVerifierAssignmentAllowedCurrencyRule {
    fn check(&self, item: &CappedVerifierAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is too long", item.currency)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierAssignmentRequiredCurrencyRule {
    pub message: String,
}

impl CappedVerifierAssignmentRule for CappedVerifierAssignmentRequiredCurrencyRule {
    fn check(&self, item: &CappedVerifierAssignment) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped verifier assignment.
#[derive(Debug, Clone)]
pub struct CappedVerifierAssignmentValidator {
    pub rules: Vec<Rc<dyn CappedVerifierAssignmentRule>>,
}

impl CappedVerifierAssignmentValidator {
    pub fn validate(&self, item: &CappedVerifierAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedVerifierAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedVerifierAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_verifier_assignment_validator() -> CappedVerifierAssignmentValidator {
    CappedVerifierAssignmentValidator {
        rules: vec![
            Rc::new(CappedVerifierAssignmentThresholdRangeRule { min: 1.1, max: 4161.2 })
                as Rc<dyn CappedVerifierAssignmentRule>,
            Rc::new(CappedVerifierAssignmentCurrencyLengthRule { max_length: 86400 }),
            Rc::new(CappedVerifierAssignmentCodeLengthRule { max_length: 86400 }),
            Rc::new(CappedVerifierAssignmentAllowedCurrencyRule {
                allowed: vec!["manual".to_string()],
            }),
            Rc::new(CappedVerifierAssignmentRequiredCurrencyRule {
                message: "capped verifier assignment needs an entry".to_string(),
            }),
        ],
    }
}
