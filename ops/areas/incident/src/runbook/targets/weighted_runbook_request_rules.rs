use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted runbook request must pass. Check returns its problems, or none.
pub trait WeightedRunbookRequestRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRunbookRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRunbookRequestRequiredCipherRule {
    pub message: String,
}

impl WeightedRunbookRequestRule for WeightedRunbookRequestRequiredCipherRule {
    fn check(&self, item: &WeightedRunbookRequest) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRunbookRequestRequiredSourceRule {
    pub message: String,
}

impl WeightedRunbookRequestRule for WeightedRunbookRequestRequiredSourceRule {
    fn check(&self, item: &WeightedRunbookRequest) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRunbookRequestRequiredCodeRule {
    pub message: String,
}

impl WeightedRunbookRequestRule for WeightedRunbookRequestRequiredCodeRule {
    fn check(&self, item: &WeightedRunbookRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRunbookRequestAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedRunbookRequestRule for WeightedRunbookRequestAttemptsRangeRule {
    fn check(&self, item: &WeightedRunbookRequest) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a weighted runbook request.
#[derive(Debug, Clone)]
pub struct WeightedRunbookRequestValidator {
    pub rules: Vec<Rc<dyn WeightedRunbookRequestRule>>,
}

impl WeightedRunbookRequestValidator {
    pub fn validate(&self, item: &WeightedRunbookRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedRunbookRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRunbookRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_runbook_request_validator() -> WeightedRunbookRequestValidator {
    WeightedRunbookRequestValidator {
        rules: vec![
            Rc::new(WeightedRunbookRequestRequiredCipherRule {
                message: "weighted runbook request needs this field".to_string(),
            }) as Rc<dyn WeightedRunbookRequestRule>,
            Rc::new(WeightedRunbookRequestRequiredSourceRule {
                message: "weighted runbook request needs an entry".to_string(),
            }),
            Rc::new(WeightedRunbookRequestRequiredCodeRule {
                message: "weighted runbook request needs an entry".to_string(),
            }),
            Rc::new(WeightedRunbookRequestAttemptsRangeRule { min: 16, max: 32 }),
        ],
    }
}
