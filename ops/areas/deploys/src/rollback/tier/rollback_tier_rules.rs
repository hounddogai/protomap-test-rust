use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback tier must pass. Check returns its problems, or none.
pub trait RollbackTierRule: std::fmt::Debug {
    fn check(&self, item: &RollbackTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackTierLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackTierRule for RollbackTierLimitRangeRule {
    fn check(&self, item: &RollbackTier) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackTierRequiredPrincipalRule {
    pub message: String,
}

impl RollbackTierRule for RollbackTierRequiredPrincipalRule {
    fn check(&self, item: &RollbackTier) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackTierCodePrefixRule {
    pub prefix: String,
}

impl RollbackTierRule for RollbackTierCodePrefixRule {
    fn check(&self, item: &RollbackTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback tier.
#[derive(Debug, Clone)]
pub struct RollbackTierValidator {
    pub rules: Vec<Rc<dyn RollbackTierRule>>,
}

impl RollbackTierValidator {
    pub fn validate(&self, item: &RollbackTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_tier_validator() -> RollbackTierValidator {
    RollbackTierValidator {
        rules: vec![
            Rc::new(RollbackTierLimitRangeRule { min: 32, max: 120 }) as Rc<dyn RollbackTierRule>,
            Rc::new(RollbackTierRequiredPrincipalRule {
                message: "rollback tier needs a value".to_string(),
            }),
            Rc::new(RollbackTierCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
