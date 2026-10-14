use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive failover exception must pass. Check returns its problems, or none.
pub trait AdaptiveFailoverExceptionRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveFailoverException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFailoverExceptionCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveFailoverExceptionRule for AdaptiveFailoverExceptionCodeLengthRule {
    fn check(&self, item: &AdaptiveFailoverException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFailoverExceptionNamespaceLengthRule {
    pub max_length: i64,
}

impl AdaptiveFailoverExceptionRule for AdaptiveFailoverExceptionNamespaceLengthRule {
    fn check(&self, item: &AdaptiveFailoverException) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFailoverExceptionDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveFailoverExceptionRule for AdaptiveFailoverExceptionDiscountRangeRule {
    fn check(&self, item: &AdaptiveFailoverException) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFailoverExceptionSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveFailoverExceptionRule for AdaptiveFailoverExceptionSequenceRangeRule {
    fn check(&self, item: &AdaptiveFailoverException) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFailoverExceptionRequiredLabelRule {
    pub message: String,
}

impl AdaptiveFailoverExceptionRule for AdaptiveFailoverExceptionRequiredLabelRule {
    fn check(&self, item: &AdaptiveFailoverException) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive failover exception.
#[derive(Debug, Clone)]
pub struct AdaptiveFailoverExceptionValidator {
    pub rules: Vec<Rc<dyn AdaptiveFailoverExceptionRule>>,
}

impl AdaptiveFailoverExceptionValidator {
    pub fn validate(&self, item: &AdaptiveFailoverException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveFailoverException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveFailoverException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_failover_exception_validator() -> AdaptiveFailoverExceptionValidator {
    AdaptiveFailoverExceptionValidator {
        rules: vec![
            Rc::new(AdaptiveFailoverExceptionCodeLengthRule { max_length: 1000 })
                as Rc<dyn AdaptiveFailoverExceptionRule>,
            Rc::new(AdaptiveFailoverExceptionNamespaceLengthRule { max_length: 1000 }),
            Rc::new(AdaptiveFailoverExceptionDiscountRangeRule { min: 6.0, max: 1565.5 }),
            Rc::new(AdaptiveFailoverExceptionSequenceRangeRule { min: 120, max: 1000 }),
            Rc::new(AdaptiveFailoverExceptionRequiredLabelRule {
                message: "adaptive failover exception needs a value".to_string(),
            }),
        ],
    }
}
