use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote adjustment must pass. Check returns its problems, or none.
pub trait QuoteAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &QuoteAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAdjustmentRequiredLabelRule {
    pub message: String,
}

impl QuoteAdjustmentRule for QuoteAdjustmentRequiredLabelRule {
    fn check(&self, item: &QuoteAdjustment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAdjustmentQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteAdjustmentRule for QuoteAdjustmentQuantityRangeRule {
    fn check(&self, item: &QuoteAdjustment) -> Vec<String> {
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
pub struct QuoteAdjustmentRequiredCodeRule {
    pub message: String,
}

impl QuoteAdjustmentRule for QuoteAdjustmentRequiredCodeRule {
    fn check(&self, item: &QuoteAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAdjustmentRequiredFingerprintRule {
    pub message: String,
}

impl QuoteAdjustmentRule for QuoteAdjustmentRequiredFingerprintRule {
    fn check(&self, item: &QuoteAdjustment) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAdjustmentRequiredNameRule {
    pub message: String,
}

impl QuoteAdjustmentRule for QuoteAdjustmentRequiredNameRule {
    fn check(&self, item: &QuoteAdjustment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAdjustmentNameLengthRule {
    pub max_length: i64,
}

impl QuoteAdjustmentRule for QuoteAdjustmentNameLengthRule {
    fn check(&self, item: &QuoteAdjustment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote adjustment.
#[derive(Debug, Clone)]
pub struct QuoteAdjustmentValidator {
    pub rules: Vec<Rc<dyn QuoteAdjustmentRule>>,
}

impl QuoteAdjustmentValidator {
    pub fn validate(&self, item: &QuoteAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuoteAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_adjustment_validator() -> QuoteAdjustmentValidator {
    QuoteAdjustmentValidator {
        rules: vec![
            Rc::new(QuoteAdjustmentRequiredLabelRule {
                message: "quote adjustment needs this field".to_string(),
            }) as Rc<dyn QuoteAdjustmentRule>,
            Rc::new(QuoteAdjustmentQuantityRangeRule { min: 120, max: 3600 }),
            Rc::new(QuoteAdjustmentRequiredCodeRule {
                message: "quote adjustment needs an entry".to_string(),
            }),
            Rc::new(QuoteAdjustmentRequiredFingerprintRule {
                message: "quote adjustment needs this field".to_string(),
            }),
            Rc::new(QuoteAdjustmentRequiredNameRule {
                message: "quote adjustment needs this field".to_string(),
            }),
            Rc::new(QuoteAdjustmentNameLengthRule { max_length: 64 }),
        ],
    }
}
