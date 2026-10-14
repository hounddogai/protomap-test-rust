use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce request must pass. Check returns its problems, or none.
pub trait NonceRequestRule: std::fmt::Debug {
    fn check(&self, item: &NonceRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceRequestMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl NonceRequestRule for NonceRequestMinimumRangeRule {
    fn check(&self, item: &NonceRequest) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceRequestRequiredLabelRule {
    pub message: String,
}

impl NonceRequestRule for NonceRequestRequiredLabelRule {
    fn check(&self, item: &NonceRequest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceRequestBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl NonceRequestRule for NonceRequestBudgetRangeRule {
    fn check(&self, item: &NonceRequest) -> Vec<String> {
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
pub struct NonceRequestCodePrefixRule {
    pub prefix: String,
}

impl NonceRequestRule for NonceRequestCodePrefixRule {
    fn check(&self, item: &NonceRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceRequestQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceRequestRule for NonceRequestQuantityRangeRule {
    fn check(&self, item: &NonceRequest) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce request.
#[derive(Debug, Clone)]
pub struct NonceRequestValidator {
    pub rules: Vec<Rc<dyn NonceRequestRule>>,
}

impl NonceRequestValidator {
    pub fn validate(&self, item: &NonceRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_request_validator() -> NonceRequestValidator {
    NonceRequestValidator {
        rules: vec![
            Rc::new(NonceRequestMinimumRangeRule { min: 8.9, max: 4482.0 }) as Rc<dyn NonceRequestRule>,
            Rc::new(NonceRequestRequiredLabelRule {
                message: "nonce request needs a value".to_string(),
            }),
            Rc::new(NonceRequestBudgetRangeRule { min: 6.7, max: 4564.4 }),
            Rc::new(NonceRequestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(NonceRequestQuantityRangeRule { min: 80, max: 255 }),
        ],
    }
}
