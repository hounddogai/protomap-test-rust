use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default tenant mapping must pass. Check returns its problems, or none.
pub trait DefaultTenantMappingRule: std::fmt::Debug {
    fn check(&self, item: &DefaultTenantMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantMappingCodeLengthRule {
    pub max_length: i64,
}

impl DefaultTenantMappingRule for DefaultTenantMappingCodeLengthRule {
    fn check(&self, item: &DefaultTenantMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantMappingTitleLengthRule {
    pub max_length: i64,
}

impl DefaultTenantMappingRule for DefaultTenantMappingTitleLengthRule {
    fn check(&self, item: &DefaultTenantMapping) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantMappingRequiredCodeRule {
    pub message: String,
}

impl DefaultTenantMappingRule for DefaultTenantMappingRequiredCodeRule {
    fn check(&self, item: &DefaultTenantMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantMappingRequiredTitleRule {
    pub message: String,
}

impl DefaultTenantMappingRule for DefaultTenantMappingRequiredTitleRule {
    fn check(&self, item: &DefaultTenantMapping) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantMappingRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultTenantMappingRule for DefaultTenantMappingRetriesRangeRule {
    fn check(&self, item: &DefaultTenantMapping) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantMappingAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl DefaultTenantMappingRule for DefaultTenantMappingAllowedTitleRule {
    fn check(&self, item: &DefaultTenantMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is too long", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default tenant mapping.
#[derive(Debug, Clone)]
pub struct DefaultTenantMappingValidator {
    pub rules: Vec<Rc<dyn DefaultTenantMappingRule>>,
}

impl DefaultTenantMappingValidator {
    pub fn validate(&self, item: &DefaultTenantMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultTenantMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultTenantMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_tenant_mapping_validator() -> DefaultTenantMappingValidator {
    DefaultTenantMappingValidator {
        rules: vec![
            Rc::new(DefaultTenantMappingCodeLengthRule { max_length: 16 }) as Rc<dyn DefaultTenantMappingRule>,
            Rc::new(DefaultTenantMappingTitleLengthRule { max_length: 3600 }),
            Rc::new(DefaultTenantMappingRequiredCodeRule {
                message: "default tenant mapping needs this field".to_string(),
            }),
            Rc::new(DefaultTenantMappingRequiredTitleRule {
                message: "default tenant mapping needs a value".to_string(),
            }),
            Rc::new(DefaultTenantMappingRetriesRangeRule { min: 120, max: 3600 }),
            Rc::new(DefaultTenantMappingAllowedTitleRule {
                allowed: vec!["weighted".to_string(), "fallback".to_string(), "rolling".to_string()],
            }),
        ],
    }
}
