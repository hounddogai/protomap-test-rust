use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation allocation must pass. Check returns its problems, or none.
pub trait EscalationAllocationRule: std::fmt::Debug {
    fn check(&self, item: &EscalationAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAllocationExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationAllocationRule for EscalationAllocationExpiresAtRangeRule {
    fn check(&self, item: &EscalationAllocation) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAllocationCodeLengthRule {
    pub max_length: i64,
}

impl EscalationAllocationRule for EscalationAllocationCodeLengthRule {
    fn check(&self, item: &EscalationAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAllocationCodePrefixRule {
    pub prefix: String,
}

impl EscalationAllocationRule for EscalationAllocationCodePrefixRule {
    fn check(&self, item: &EscalationAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation allocation.
#[derive(Debug, Clone)]
pub struct EscalationAllocationValidator {
    pub rules: Vec<Rc<dyn EscalationAllocationRule>>,
}

impl EscalationAllocationValidator {
    pub fn validate(&self, item: &EscalationAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EscalationAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_allocation_validator() -> EscalationAllocationValidator {
    EscalationAllocationValidator {
        rules: vec![
            Rc::new(EscalationAllocationExpiresAtRangeRule { min: 32, max: 64 }) as Rc<dyn EscalationAllocationRule>,
            Rc::new(EscalationAllocationCodeLengthRule { max_length: 80 }),
            Rc::new(EscalationAllocationCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
