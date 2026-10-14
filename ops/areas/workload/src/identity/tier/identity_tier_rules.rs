use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a identity tier must pass. Check returns its problems, or none.
pub trait IdentityTierRule: std::fmt::Debug {
    fn check(&self, item: &IdentityTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTierActiveGuardRule {
    pub limit: i64,
}

impl IdentityTierRule for IdentityTierActiveGuardRule {
    fn check(&self, item: &IdentityTier) -> Vec<String> {
        if item.active && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while active",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTierAllowedHostRule {
    pub allowed: Vec<String>,
}

impl IdentityTierRule for IdentityTierAllowedHostRule {
    fn check(&self, item: &IdentityTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is too long", item.host)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTierRequiredCodeRule {
    pub message: String,
}

impl IdentityTierRule for IdentityTierRequiredCodeRule {
    fn check(&self, item: &IdentityTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTierRequiredNameRule {
    pub message: String,
}

impl IdentityTierRule for IdentityTierRequiredNameRule {
    fn check(&self, item: &IdentityTier) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTierNameLengthRule {
    pub max_length: i64,
}

impl IdentityTierRule for IdentityTierNameLengthRule {
    fn check(&self, item: &IdentityTier) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity tier.
#[derive(Debug, Clone)]
pub struct IdentityTierValidator {
    pub rules: Vec<Rc<dyn IdentityTierRule>>,
}

impl IdentityTierValidator {
    pub fn validate(&self, item: &IdentityTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IdentityTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_tier_validator() -> IdentityTierValidator {
    IdentityTierValidator {
        rules: vec![
            Rc::new(IdentityTierActiveGuardRule { limit: 86400 }) as Rc<dyn IdentityTierRule>,
            Rc::new(IdentityTierAllowedHostRule {
                allowed: vec!["pending".to_string(), "default".to_string(), "lenient".to_string()],
            }),
            Rc::new(IdentityTierRequiredCodeRule {
                message: "identity tier needs an entry".to_string(),
            }),
            Rc::new(IdentityTierRequiredNameRule {
                message: "identity tier needs this field".to_string(),
            }),
            Rc::new(IdentityTierNameLengthRule { max_length: 16 }),
        ],
    }
}
