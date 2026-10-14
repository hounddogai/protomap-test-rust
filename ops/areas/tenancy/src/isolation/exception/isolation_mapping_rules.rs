use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a isolation mapping must pass. Check returns its problems, or none.
pub trait IsolationMappingRule: std::fmt::Debug {
    fn check(&self, item: &IsolationMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationMappingCodePrefixRule {
    pub prefix: String,
}

impl IsolationMappingRule for IsolationMappingCodePrefixRule {
    fn check(&self, item: &IsolationMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationMappingAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl IsolationMappingRule for IsolationMappingAllowedKeyIdRule {
    fn check(&self, item: &IsolationMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is above maximum", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationMappingRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationMappingRule for IsolationMappingRevisionRangeRule {
    fn check(&self, item: &IsolationMapping) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationMappingQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationMappingRule for IsolationMappingQuantityRangeRule {
    fn check(&self, item: &IsolationMapping) -> Vec<String> {
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
pub struct IsolationMappingLabelLengthRule {
    pub max_length: i64,
}

impl IsolationMappingRule for IsolationMappingLabelLengthRule {
    fn check(&self, item: &IsolationMapping) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation mapping.
#[derive(Debug, Clone)]
pub struct IsolationMappingValidator {
    pub rules: Vec<Rc<dyn IsolationMappingRule>>,
}

impl IsolationMappingValidator {
    pub fn validate(&self, item: &IsolationMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_mapping_validator() -> IsolationMappingValidator {
    IsolationMappingValidator {
        rules: vec![
            Rc::new(IsolationMappingCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn IsolationMappingRule>,
            Rc::new(IsolationMappingAllowedKeyIdRule {
                allowed: vec!["capped".to_string(), "adaptive".to_string(), "capped".to_string()],
            }),
            Rc::new(IsolationMappingRevisionRangeRule { min: 86400, max: 86400 }),
            Rc::new(IsolationMappingQuantityRangeRule { min: 32, max: 3600 }),
            Rc::new(IsolationMappingLabelLengthRule { max_length: 32 }),
        ],
    }
}
