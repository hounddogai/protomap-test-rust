use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped region pin tier must pass. Check returns its problems, or none.
pub trait CappedRegionPinTierRule: std::fmt::Debug {
    fn check(&self, item: &CappedRegionPinTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinTierRequiredCodeRule {
    pub message: String,
}

impl CappedRegionPinTierRule for CappedRegionPinTierRequiredCodeRule {
    fn check(&self, item: &CappedRegionPinTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinTierCodePrefixRule {
    pub prefix: String,
}

impl CappedRegionPinTierRule for CappedRegionPinTierCodePrefixRule {
    fn check(&self, item: &CappedRegionPinTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinTierRequiredNotesRule {
    pub message: String,
}

impl CappedRegionPinTierRule for CappedRegionPinTierRequiredNotesRule {
    fn check(&self, item: &CappedRegionPinTier) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinTierCodeLengthRule {
    pub max_length: i64,
}

impl CappedRegionPinTierRule for CappedRegionPinTierCodeLengthRule {
    fn check(&self, item: &CappedRegionPinTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinTierVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedRegionPinTierRule for CappedRegionPinTierVersionRangeRule {
    fn check(&self, item: &CappedRegionPinTier) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinTierRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedRegionPinTierRule for CappedRegionPinTierRetriesRangeRule {
    fn check(&self, item: &CappedRegionPinTier) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped region pin tier.
#[derive(Debug, Clone)]
pub struct CappedRegionPinTierValidator {
    pub rules: Vec<Rc<dyn CappedRegionPinTierRule>>,
}

impl CappedRegionPinTierValidator {
    pub fn validate(&self, item: &CappedRegionPinTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedRegionPinTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedRegionPinTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_region_pin_tier_validator() -> CappedRegionPinTierValidator {
    CappedRegionPinTierValidator {
        rules: vec![
            Rc::new(CappedRegionPinTierRequiredCodeRule {
                message: "capped region pin tier needs an entry".to_string(),
            }) as Rc<dyn CappedRegionPinTierRule>,
            Rc::new(CappedRegionPinTierCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(CappedRegionPinTierRequiredNotesRule {
                message: "capped region pin tier needs an entry".to_string(),
            }),
            Rc::new(CappedRegionPinTierCodeLengthRule { max_length: 3600 }),
            Rc::new(CappedRegionPinTierVersionRangeRule { min: 32, max: 64 }),
            Rc::new(CappedRegionPinTierRetriesRangeRule { min: 16, max: 120 }),
        ],
    }
}
