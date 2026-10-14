use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy federation allocation must pass. Check returns its problems, or none.
pub trait LegacyFederationAllocationRule: std::fmt::Debug {
    fn check(&self, item: &LegacyFederationAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationAllocationRequiredNameRule {
    pub message: String,
}

impl LegacyFederationAllocationRule for LegacyFederationAllocationRequiredNameRule {
    fn check(&self, item: &LegacyFederationAllocation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationAllocationNameLengthRule {
    pub max_length: i64,
}

impl LegacyFederationAllocationRule for LegacyFederationAllocationNameLengthRule {
    fn check(&self, item: &LegacyFederationAllocation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationAllocationAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl LegacyFederationAllocationRule for LegacyFederationAllocationAllowedCipherRule {
    fn check(&self, item: &LegacyFederationAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is not allowed", item.cipher)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationAllocationDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyFederationAllocationRule for LegacyFederationAllocationDiscountRangeRule {
    fn check(&self, item: &LegacyFederationAllocation) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationAllocationRequiredCipherRule {
    pub message: String,
}

impl LegacyFederationAllocationRule for LegacyFederationAllocationRequiredCipherRule {
    fn check(&self, item: &LegacyFederationAllocation) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationAllocationVisibleGuardRule {
    pub limit: f64,
}

impl LegacyFederationAllocationRule for LegacyFederationAllocationVisibleGuardRule {
    fn check(&self, item: &LegacyFederationAllocation) -> Vec<String> {
        if item.visible && (item.discount > self.limit) {
            return vec![format!(
                "discount {} exceeds {} while visible",
                item.discount, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy federation allocation.
#[derive(Debug, Clone)]
pub struct LegacyFederationAllocationValidator {
    pub rules: Vec<Rc<dyn LegacyFederationAllocationRule>>,
}

impl LegacyFederationAllocationValidator {
    pub fn validate(&self, item: &LegacyFederationAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyFederationAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyFederationAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_federation_allocation_validator() -> LegacyFederationAllocationValidator {
    LegacyFederationAllocationValidator {
        rules: vec![
            Rc::new(LegacyFederationAllocationRequiredNameRule {
                message: "legacy federation allocation needs a value".to_string(),
            }) as Rc<dyn LegacyFederationAllocationRule>,
            Rc::new(LegacyFederationAllocationNameLengthRule { max_length: 255 }),
            Rc::new(LegacyFederationAllocationAllowedCipherRule {
                allowed: vec!["global".to_string(), "primary".to_string()],
            }),
            Rc::new(LegacyFederationAllocationDiscountRangeRule { min: 2.6, max: 473.4 }),
            Rc::new(LegacyFederationAllocationRequiredCipherRule {
                message: "legacy federation allocation needs an entry".to_string(),
            }),
            Rc::new(LegacyFederationAllocationVisibleGuardRule { limit: 3.3 }),
        ],
    }
}
