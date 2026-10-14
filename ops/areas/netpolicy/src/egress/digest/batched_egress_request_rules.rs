use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched egress request must pass. Check returns its problems, or none.
pub trait BatchedEgressRequestRule: std::fmt::Debug {
    fn check(&self, item: &BatchedEgressRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEgressRequestTitleLengthRule {
    pub max_length: i64,
}

impl BatchedEgressRequestRule for BatchedEgressRequestTitleLengthRule {
    fn check(&self, item: &BatchedEgressRequest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEgressRequestAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl BatchedEgressRequestRule for BatchedEgressRequestAllowedPrincipalRule {
    fn check(&self, item: &BatchedEgressRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is unknown", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEgressRequestWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedEgressRequestRule for BatchedEgressRequestWeightRangeRule {
    fn check(&self, item: &BatchedEgressRequest) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEgressRequestRequiredCodeRule {
    pub message: String,
}

impl BatchedEgressRequestRule for BatchedEgressRequestRequiredCodeRule {
    fn check(&self, item: &BatchedEgressRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEgressRequestCodePrefixRule {
    pub prefix: String,
}

impl BatchedEgressRequestRule for BatchedEgressRequestCodePrefixRule {
    fn check(&self, item: &BatchedEgressRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEgressRequestCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedEgressRequestRule for BatchedEgressRequestCeilingRangeRule {
    fn check(&self, item: &BatchedEgressRequest) -> Vec<String> {
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

/// Runs every rule of a batched egress request.
#[derive(Debug, Clone)]
pub struct BatchedEgressRequestValidator {
    pub rules: Vec<Rc<dyn BatchedEgressRequestRule>>,
}

impl BatchedEgressRequestValidator {
    pub fn validate(&self, item: &BatchedEgressRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedEgressRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedEgressRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_egress_request_validator() -> BatchedEgressRequestValidator {
    BatchedEgressRequestValidator {
        rules: vec![
            Rc::new(BatchedEgressRequestTitleLengthRule { max_length: 32 }) as Rc<dyn BatchedEgressRequestRule>,
            Rc::new(BatchedEgressRequestAllowedPrincipalRule { allowed: Vec::new() }),
            Rc::new(BatchedEgressRequestWeightRangeRule { min: 8.9, max: 3451.9 }),
            Rc::new(BatchedEgressRequestRequiredCodeRule {
                message: "batched egress request needs a value".to_string(),
            }),
            Rc::new(BatchedEgressRequestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(BatchedEgressRequestCeilingRangeRule { min: 3.5, max: 2256.1 }),
        ],
    }
}
