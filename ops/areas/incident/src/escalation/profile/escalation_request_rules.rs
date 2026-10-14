use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation request must pass. Check returns its problems, or none.
pub trait EscalationRequestRule: std::fmt::Debug {
    fn check(&self, item: &EscalationRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationRequestCodePrefixRule {
    pub prefix: String,
}

impl EscalationRequestRule for EscalationRequestCodePrefixRule {
    fn check(&self, item: &EscalationRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationRequestStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationRequestRule for EscalationRequestStartsAtRangeRule {
    fn check(&self, item: &EscalationRequest) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationRequestPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationRequestRule for EscalationRequestPortRangeRule {
    fn check(&self, item: &EscalationRequest) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationRequestRequiredTitleRule {
    pub message: String,
}

impl EscalationRequestRule for EscalationRequestRequiredTitleRule {
    fn check(&self, item: &EscalationRequest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation request.
#[derive(Debug, Clone)]
pub struct EscalationRequestValidator {
    pub rules: Vec<Rc<dyn EscalationRequestRule>>,
}

impl EscalationRequestValidator {
    pub fn validate(&self, item: &EscalationRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EscalationRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_request_validator() -> EscalationRequestValidator {
    EscalationRequestValidator {
        rules: vec![
            Rc::new(EscalationRequestCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn EscalationRequestRule>,
            Rc::new(EscalationRequestStartsAtRangeRule { min: 16, max: 1000 }),
            Rc::new(EscalationRequestPortRangeRule { min: 80, max: 255 }),
            Rc::new(EscalationRequestRequiredTitleRule {
                message: "escalation request needs this field".to_string(),
            }),
        ],
    }
}
