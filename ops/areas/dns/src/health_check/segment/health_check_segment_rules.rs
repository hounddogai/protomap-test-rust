use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check segment must pass. Check returns its problems, or none.
pub trait HealthCheckSegmentRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckSegmentRequiredCountryRule {
    pub message: String,
}

impl HealthCheckSegmentRule for HealthCheckSegmentRequiredCountryRule {
    fn check(&self, item: &HealthCheckSegment) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckSegmentCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckSegmentRule for HealthCheckSegmentCeilingRangeRule {
    fn check(&self, item: &HealthCheckSegment) -> Vec<String> {
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
pub struct HealthCheckSegmentRequiredCipherRule {
    pub message: String,
}

impl HealthCheckSegmentRule for HealthCheckSegmentRequiredCipherRule {
    fn check(&self, item: &HealthCheckSegment) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckSegmentCodePrefixRule {
    pub prefix: String,
}

impl HealthCheckSegmentRule for HealthCheckSegmentCodePrefixRule {
    fn check(&self, item: &HealthCheckSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckSegmentCodeLengthRule {
    pub max_length: i64,
}

impl HealthCheckSegmentRule for HealthCheckSegmentCodeLengthRule {
    fn check(&self, item: &HealthCheckSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckSegmentNameLengthRule {
    pub max_length: i64,
}

impl HealthCheckSegmentRule for HealthCheckSegmentNameLengthRule {
    fn check(&self, item: &HealthCheckSegment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check segment.
#[derive(Debug, Clone)]
pub struct HealthCheckSegmentValidator {
    pub rules: Vec<Rc<dyn HealthCheckSegmentRule>>,
}

impl HealthCheckSegmentValidator {
    pub fn validate(&self, item: &HealthCheckSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_segment_validator() -> HealthCheckSegmentValidator {
    HealthCheckSegmentValidator {
        rules: vec![
            Rc::new(HealthCheckSegmentRequiredCountryRule {
                message: "health check segment needs an entry".to_string(),
            }) as Rc<dyn HealthCheckSegmentRule>,
            Rc::new(HealthCheckSegmentCeilingRangeRule { min: 8.1, max: 3981.0 }),
            Rc::new(HealthCheckSegmentRequiredCipherRule {
                message: "health check segment needs a value".to_string(),
            }),
            Rc::new(HealthCheckSegmentCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(HealthCheckSegmentCodeLengthRule { max_length: 64 }),
            Rc::new(HealthCheckSegmentNameLengthRule { max_length: 86400 }),
        ],
    }
}
