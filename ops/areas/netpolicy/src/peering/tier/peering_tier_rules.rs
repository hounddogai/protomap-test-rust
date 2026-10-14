use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a peering tier must pass. Check returns its problems, or none.
pub trait PeeringTierRule: std::fmt::Debug {
    fn check(&self, item: &PeeringTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringTierCodeLengthRule {
    pub max_length: i64,
}

impl PeeringTierRule for PeeringTierCodeLengthRule {
    fn check(&self, item: &PeeringTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringTierAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PeeringTierRule for PeeringTierAllowedLabelRule {
    fn check(&self, item: &PeeringTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is duplicated", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringTierRequiredUnitRule {
    pub message: String,
}

impl PeeringTierRule for PeeringTierRequiredUnitRule {
    fn check(&self, item: &PeeringTier) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringTierRequiredLabelRule {
    pub message: String,
}

impl PeeringTierRule for PeeringTierRequiredLabelRule {
    fn check(&self, item: &PeeringTier) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringTierUnitLengthRule {
    pub max_length: i64,
}

impl PeeringTierRule for PeeringTierUnitLengthRule {
    fn check(&self, item: &PeeringTier) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringTierAudienceLengthRule {
    pub max_length: i64,
}

impl PeeringTierRule for PeeringTierAudienceLengthRule {
    fn check(&self, item: &PeeringTier) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering tier.
#[derive(Debug, Clone)]
pub struct PeeringTierValidator {
    pub rules: Vec<Rc<dyn PeeringTierRule>>,
}

impl PeeringTierValidator {
    pub fn validate(&self, item: &PeeringTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PeeringTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_tier_validator() -> PeeringTierValidator {
    PeeringTierValidator {
        rules: vec![
            Rc::new(PeeringTierCodeLengthRule { max_length: 64 }) as Rc<dyn PeeringTierRule>,
            Rc::new(PeeringTierAllowedLabelRule {
                allowed: vec!["manual".to_string(), "adaptive".to_string()],
            }),
            Rc::new(PeeringTierRequiredUnitRule {
                message: "peering tier needs this field".to_string(),
            }),
            Rc::new(PeeringTierRequiredLabelRule {
                message: "peering tier needs a value".to_string(),
            }),
            Rc::new(PeeringTierUnitLengthRule { max_length: 120 }),
            Rc::new(PeeringTierAudienceLengthRule { max_length: 16 }),
        ],
    }
}
