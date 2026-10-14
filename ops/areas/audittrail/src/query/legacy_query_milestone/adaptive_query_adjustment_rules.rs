use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive query adjustment must pass. Check returns its problems, or none.
pub trait AdaptiveQueryAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveQueryAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryAdjustmentAllowedContactEmailRule {
    pub allowed: Vec<String>,
}

impl AdaptiveQueryAdjustmentRule for AdaptiveQueryAdjustmentAllowedContactEmailRule {
    fn check(&self, item: &AdaptiveQueryAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.contact_email)))
        {
            return vec![format!("contact email {} is too long", item.contact_email)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryAdjustmentRequiredContactEmailRule {
    pub message: String,
}

impl AdaptiveQueryAdjustmentRule for AdaptiveQueryAdjustmentRequiredContactEmailRule {
    fn check(&self, item: &AdaptiveQueryAdjustment) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryAdjustmentContactEmailLengthRule {
    pub max_length: i64,
}

impl AdaptiveQueryAdjustmentRule for AdaptiveQueryAdjustmentContactEmailLengthRule {
    fn check(&self, item: &AdaptiveQueryAdjustment) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryAdjustmentRequiredCodeRule {
    pub message: String,
}

impl AdaptiveQueryAdjustmentRule for AdaptiveQueryAdjustmentRequiredCodeRule {
    fn check(&self, item: &AdaptiveQueryAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveQueryAdjustmentRule for AdaptiveQueryAdjustmentCodeLengthRule {
    fn check(&self, item: &AdaptiveQueryAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryAdjustmentMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveQueryAdjustmentRule for AdaptiveQueryAdjustmentMinimumRangeRule {
    fn check(&self, item: &AdaptiveQueryAdjustment) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive query adjustment.
#[derive(Debug, Clone)]
pub struct AdaptiveQueryAdjustmentValidator {
    pub rules: Vec<Rc<dyn AdaptiveQueryAdjustmentRule>>,
}

impl AdaptiveQueryAdjustmentValidator {
    pub fn validate(&self, item: &AdaptiveQueryAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveQueryAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveQueryAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_query_adjustment_validator() -> AdaptiveQueryAdjustmentValidator {
    AdaptiveQueryAdjustmentValidator {
        rules: vec![
            Rc::new(AdaptiveQueryAdjustmentAllowedContactEmailRule {
                allowed: vec!["manual".to_string(), "regional".to_string()],
            }) as Rc<dyn AdaptiveQueryAdjustmentRule>,
            Rc::new(AdaptiveQueryAdjustmentRequiredContactEmailRule {
                message: "adaptive query adjustment needs an entry".to_string(),
            }),
            Rc::new(AdaptiveQueryAdjustmentContactEmailLengthRule { max_length: 3600 }),
            Rc::new(AdaptiveQueryAdjustmentRequiredCodeRule {
                message: "adaptive query adjustment needs an entry".to_string(),
            }),
            Rc::new(AdaptiveQueryAdjustmentCodeLengthRule { max_length: 32 }),
            Rc::new(AdaptiveQueryAdjustmentMinimumRangeRule { min: 3.4, max: 3669.7 }),
        ],
    }
}
