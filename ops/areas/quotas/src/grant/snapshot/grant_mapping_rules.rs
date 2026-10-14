use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant mapping must pass. Check returns its problems, or none.
pub trait GrantMappingRule: std::fmt::Debug {
    fn check(&self, item: &GrantMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantMappingRequiredTenantIdRule {
    pub message: String,
}

impl GrantMappingRule for GrantMappingRequiredTenantIdRule {
    fn check(&self, item: &GrantMapping) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantMappingDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GrantMappingRule for GrantMappingDiscountRangeRule {
    fn check(&self, item: &GrantMapping) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantMappingLabelLengthRule {
    pub max_length: i64,
}

impl GrantMappingRule for GrantMappingLabelLengthRule {
    fn check(&self, item: &GrantMapping) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantMappingTenantIdLengthRule {
    pub max_length: i64,
}

impl GrantMappingRule for GrantMappingTenantIdLengthRule {
    fn check(&self, item: &GrantMapping) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant mapping.
#[derive(Debug, Clone)]
pub struct GrantMappingValidator {
    pub rules: Vec<Rc<dyn GrantMappingRule>>,
}

impl GrantMappingValidator {
    pub fn validate(&self, item: &GrantMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GrantMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_mapping_validator() -> GrantMappingValidator {
    GrantMappingValidator {
        rules: vec![
            Rc::new(GrantMappingRequiredTenantIdRule {
                message: "grant mapping needs a value".to_string(),
            }) as Rc<dyn GrantMappingRule>,
            Rc::new(GrantMappingDiscountRangeRule { min: 0.1, max: 3484.5 }),
            Rc::new(GrantMappingLabelLengthRule { max_length: 1000 }),
            Rc::new(GrantMappingTenantIdLengthRule { max_length: 32 }),
        ],
    }
}
