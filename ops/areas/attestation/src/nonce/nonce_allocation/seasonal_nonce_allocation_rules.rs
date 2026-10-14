use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal nonce allocation must pass. Check returns its problems, or none.
pub trait SeasonalNonceAllocationRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalNonceAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalNonceAllocationExternalRefLengthRule {
    pub max_length: i64,
}

impl SeasonalNonceAllocationRule for SeasonalNonceAllocationExternalRefLengthRule {
    fn check(&self, item: &SeasonalNonceAllocation) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalNonceAllocationRequiredCodeRule {
    pub message: String,
}

impl SeasonalNonceAllocationRule for SeasonalNonceAllocationRequiredCodeRule {
    fn check(&self, item: &SeasonalNonceAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalNonceAllocationRequiredExternalRefRule {
    pub message: String,
}

impl SeasonalNonceAllocationRule for SeasonalNonceAllocationRequiredExternalRefRule {
    fn check(&self, item: &SeasonalNonceAllocation) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalNonceAllocationDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalNonceAllocationRule for SeasonalNonceAllocationDurationMinutesRangeRule {
    fn check(&self, item: &SeasonalNonceAllocation) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalNonceAllocationSegmentLengthRule {
    pub max_length: i64,
}

impl SeasonalNonceAllocationRule for SeasonalNonceAllocationSegmentLengthRule {
    fn check(&self, item: &SeasonalNonceAllocation) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalNonceAllocationAllowedClusterRule {
    pub allowed: Vec<String>,
}

impl SeasonalNonceAllocationRule for SeasonalNonceAllocationAllowedClusterRule {
    fn check(&self, item: &SeasonalNonceAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.cluster)))
        {
            return vec![format!("cluster {} is below minimum", item.cluster)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal nonce allocation.
#[derive(Debug, Clone)]
pub struct SeasonalNonceAllocationValidator {
    pub rules: Vec<Rc<dyn SeasonalNonceAllocationRule>>,
}

impl SeasonalNonceAllocationValidator {
    pub fn validate(&self, item: &SeasonalNonceAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalNonceAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalNonceAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_nonce_allocation_validator() -> SeasonalNonceAllocationValidator {
    SeasonalNonceAllocationValidator {
        rules: vec![
            Rc::new(SeasonalNonceAllocationExternalRefLengthRule { max_length: 64 })
                as Rc<dyn SeasonalNonceAllocationRule>,
            Rc::new(SeasonalNonceAllocationRequiredCodeRule {
                message: "seasonal nonce allocation needs an entry".to_string(),
            }),
            Rc::new(SeasonalNonceAllocationRequiredExternalRefRule {
                message: "seasonal nonce allocation needs an entry".to_string(),
            }),
            Rc::new(SeasonalNonceAllocationDurationMinutesRangeRule { min: 120, max: 3600 }),
            Rc::new(SeasonalNonceAllocationSegmentLengthRule { max_length: 86400 }),
            Rc::new(SeasonalNonceAllocationAllowedClusterRule {
                allowed: vec!["seasonal".to_string(), "seasonal".to_string(), "default".to_string()],
            }),
        ],
    }
}
