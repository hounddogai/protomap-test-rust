use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling limit override must pass. Check returns its problems, or none.
pub trait RollingLimitOverrideRule: std::fmt::Debug {
    fn check(&self, item: &RollingLimitOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitOverrideCodePrefixRule {
    pub prefix: String,
}

impl RollingLimitOverrideRule for RollingLimitOverrideCodePrefixRule {
    fn check(&self, item: &RollingLimitOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitOverrideRegionLengthRule {
    pub max_length: i64,
}

impl RollingLimitOverrideRule for RollingLimitOverrideRegionLengthRule {
    fn check(&self, item: &RollingLimitOverride) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitOverridePrincipalLengthRule {
    pub max_length: i64,
}

impl RollingLimitOverrideRule for RollingLimitOverridePrincipalLengthRule {
    fn check(&self, item: &RollingLimitOverride) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitOverrideRequiredRegionRule {
    pub message: String,
}

impl RollingLimitOverrideRule for RollingLimitOverrideRequiredRegionRule {
    fn check(&self, item: &RollingLimitOverride) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling limit override.
#[derive(Debug, Clone)]
pub struct RollingLimitOverrideValidator {
    pub rules: Vec<Rc<dyn RollingLimitOverrideRule>>,
}

impl RollingLimitOverrideValidator {
    pub fn validate(&self, item: &RollingLimitOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingLimitOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingLimitOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_limit_override_validator() -> RollingLimitOverrideValidator {
    RollingLimitOverrideValidator {
        rules: vec![
            Rc::new(RollingLimitOverrideCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn RollingLimitOverrideRule>,
            Rc::new(RollingLimitOverrideRegionLengthRule { max_length: 64 }),
            Rc::new(RollingLimitOverridePrincipalLengthRule { max_length: 255 }),
            Rc::new(RollingLimitOverrideRequiredRegionRule {
                message: "rolling limit override needs an entry".to_string(),
            }),
        ],
    }
}
