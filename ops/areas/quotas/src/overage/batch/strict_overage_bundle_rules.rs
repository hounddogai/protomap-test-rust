use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict overage bundle must pass. Check returns its problems, or none.
pub trait StrictOverageBundleRule: std::fmt::Debug {
    fn check(&self, item: &StrictOverageBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageBundleRequiredCodeRule {
    pub message: String,
}

impl StrictOverageBundleRule for StrictOverageBundleRequiredCodeRule {
    fn check(&self, item: &StrictOverageBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageBundleLabelLengthRule {
    pub max_length: i64,
}

impl StrictOverageBundleRule for StrictOverageBundleLabelLengthRule {
    fn check(&self, item: &StrictOverageBundle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageBundleTenantIdLengthRule {
    pub max_length: i64,
}

impl StrictOverageBundleRule for StrictOverageBundleTenantIdLengthRule {
    fn check(&self, item: &StrictOverageBundle) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageBundleCodePrefixRule {
    pub prefix: String,
}

impl StrictOverageBundleRule for StrictOverageBundleCodePrefixRule {
    fn check(&self, item: &StrictOverageBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageBundleLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictOverageBundleRule for StrictOverageBundleLimitRangeRule {
    fn check(&self, item: &StrictOverageBundle) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageBundleRequiredTenantIdRule {
    pub message: String,
}

impl StrictOverageBundleRule for StrictOverageBundleRequiredTenantIdRule {
    fn check(&self, item: &StrictOverageBundle) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict overage bundle.
#[derive(Debug, Clone)]
pub struct StrictOverageBundleValidator {
    pub rules: Vec<Rc<dyn StrictOverageBundleRule>>,
}

impl StrictOverageBundleValidator {
    pub fn validate(&self, item: &StrictOverageBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictOverageBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictOverageBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_overage_bundle_validator() -> StrictOverageBundleValidator {
    StrictOverageBundleValidator {
        rules: vec![
            Rc::new(StrictOverageBundleRequiredCodeRule {
                message: "strict overage bundle needs an entry".to_string(),
            }) as Rc<dyn StrictOverageBundleRule>,
            Rc::new(StrictOverageBundleLabelLengthRule { max_length: 3600 }),
            Rc::new(StrictOverageBundleTenantIdLengthRule { max_length: 120 }),
            Rc::new(StrictOverageBundleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(StrictOverageBundleLimitRangeRule { min: 64, max: 64 }),
            Rc::new(StrictOverageBundleRequiredTenantIdRule {
                message: "strict overage bundle needs this field".to_string(),
            }),
        ],
    }
}
