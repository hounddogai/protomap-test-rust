use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive delegation request must pass. Check returns its problems, or none.
pub trait AdaptiveDelegationRequestRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveDelegationRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationRequestRequiredCodeRule {
    pub message: String,
}

impl AdaptiveDelegationRequestRule for AdaptiveDelegationRequestRequiredCodeRule {
    fn check(&self, item: &AdaptiveDelegationRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationRequestArchivedGuardRule {
    pub limit: f64,
}

impl AdaptiveDelegationRequestRule for AdaptiveDelegationRequestArchivedGuardRule {
    fn check(&self, item: &AdaptiveDelegationRequest) -> Vec<String> {
        if item.archived && (item.ceiling > self.limit) {
            return vec![format!(
                "ceiling {} exceeds {} while archived",
                item.ceiling, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationRequestCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveDelegationRequestRule for AdaptiveDelegationRequestCodePrefixRule {
    fn check(&self, item: &AdaptiveDelegationRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationRequestCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveDelegationRequestRule for AdaptiveDelegationRequestCodeLengthRule {
    fn check(&self, item: &AdaptiveDelegationRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationRequestCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveDelegationRequestRule for AdaptiveDelegationRequestCeilingRangeRule {
    fn check(&self, item: &AdaptiveDelegationRequest) -> Vec<String> {
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
pub struct AdaptiveDelegationRequestTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveDelegationRequestRule for AdaptiveDelegationRequestTtlSecondsRangeRule {
    fn check(&self, item: &AdaptiveDelegationRequest) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive delegation request.
#[derive(Debug, Clone)]
pub struct AdaptiveDelegationRequestValidator {
    pub rules: Vec<Rc<dyn AdaptiveDelegationRequestRule>>,
}

impl AdaptiveDelegationRequestValidator {
    pub fn validate(&self, item: &AdaptiveDelegationRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveDelegationRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveDelegationRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_delegation_request_validator() -> AdaptiveDelegationRequestValidator {
    AdaptiveDelegationRequestValidator {
        rules: vec![
            Rc::new(AdaptiveDelegationRequestRequiredCodeRule {
                message: "adaptive delegation request needs a value".to_string(),
            }) as Rc<dyn AdaptiveDelegationRequestRule>,
            Rc::new(AdaptiveDelegationRequestArchivedGuardRule { limit: 2.8 }),
            Rc::new(AdaptiveDelegationRequestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(AdaptiveDelegationRequestCodeLengthRule { max_length: 32 }),
            Rc::new(AdaptiveDelegationRequestCeilingRangeRule { min: 2.6, max: 4527.0 }),
            Rc::new(AdaptiveDelegationRequestTtlSecondsRangeRule { min: 64, max: 80 }),
        ],
    }
}
