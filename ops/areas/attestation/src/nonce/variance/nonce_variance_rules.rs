use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce variance must pass. Check returns its problems, or none.
pub trait NonceVarianceRule: std::fmt::Debug {
    fn check(&self, item: &NonceVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceVarianceQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceVarianceRule for NonceVarianceQuantityRangeRule {
    fn check(&self, item: &NonceVariance) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceVarianceKeyIdLengthRule {
    pub max_length: i64,
}

impl NonceVarianceRule for NonceVarianceKeyIdLengthRule {
    fn check(&self, item: &NonceVariance) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceVarianceRequiredCipherRule {
    pub message: String,
}

impl NonceVarianceRule for NonceVarianceRequiredCipherRule {
    fn check(&self, item: &NonceVariance) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceVarianceCipherLengthRule {
    pub max_length: i64,
}

impl NonceVarianceRule for NonceVarianceCipherLengthRule {
    fn check(&self, item: &NonceVariance) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceVarianceCodeLengthRule {
    pub max_length: i64,
}

impl NonceVarianceRule for NonceVarianceCodeLengthRule {
    fn check(&self, item: &NonceVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce variance.
#[derive(Debug, Clone)]
pub struct NonceVarianceValidator {
    pub rules: Vec<Rc<dyn NonceVarianceRule>>,
}

impl NonceVarianceValidator {
    pub fn validate(&self, item: &NonceVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &NonceVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_variance_validator() -> NonceVarianceValidator {
    NonceVarianceValidator {
        rules: vec![
            Rc::new(NonceVarianceQuantityRangeRule { min: 255, max: 1000 }) as Rc<dyn NonceVarianceRule>,
            Rc::new(NonceVarianceKeyIdLengthRule { max_length: 32 }),
            Rc::new(NonceVarianceRequiredCipherRule {
                message: "nonce variance needs an entry".to_string(),
            }),
            Rc::new(NonceVarianceCipherLengthRule { max_length: 80 }),
            Rc::new(NonceVarianceCodeLengthRule { max_length: 16 }),
        ],
    }
}
