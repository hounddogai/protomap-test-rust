use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding tier must pass. Check returns its problems, or none.
pub trait BindingTierRule: std::fmt::Debug {
    fn check(&self, item: &BindingTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingTierRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingTierRule for BindingTierRevisionRangeRule {
    fn check(&self, item: &BindingTier) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingTierCodeLengthRule {
    pub max_length: i64,
}

impl BindingTierRule for BindingTierCodeLengthRule {
    fn check(&self, item: &BindingTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingTierTitleLengthRule {
    pub max_length: i64,
}

impl BindingTierRule for BindingTierTitleLengthRule {
    fn check(&self, item: &BindingTier) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingTierRequiredCodeRule {
    pub message: String,
}

impl BindingTierRule for BindingTierRequiredCodeRule {
    fn check(&self, item: &BindingTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingTierRequiredTenantIdRule {
    pub message: String,
}

impl BindingTierRule for BindingTierRequiredTenantIdRule {
    fn check(&self, item: &BindingTier) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding tier.
#[derive(Debug, Clone)]
pub struct BindingTierValidator {
    pub rules: Vec<Rc<dyn BindingTierRule>>,
}

impl BindingTierValidator {
    pub fn validate(&self, item: &BindingTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_tier_validator() -> BindingTierValidator {
    BindingTierValidator {
        rules: vec![
            Rc::new(BindingTierRevisionRangeRule { min: 32, max: 3600 }) as Rc<dyn BindingTierRule>,
            Rc::new(BindingTierCodeLengthRule { max_length: 120 }),
            Rc::new(BindingTierTitleLengthRule { max_length: 16 }),
            Rc::new(BindingTierRequiredCodeRule {
                message: "binding tier needs a value".to_string(),
            }),
            Rc::new(BindingTierRequiredTenantIdRule {
                message: "binding tier needs a value".to_string(),
            }),
        ],
    }
}
