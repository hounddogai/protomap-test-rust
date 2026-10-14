use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient region pin tier must pass. Check returns its problems, or none.
pub trait LenientRegionPinTierRule: std::fmt::Debug {
    fn check(&self, item: &LenientRegionPinTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRegionPinTierActiveGuardRule {
    pub limit: i64,
}

impl LenientRegionPinTierRule for LenientRegionPinTierActiveGuardRule {
    fn check(&self, item: &LenientRegionPinTier) -> Vec<String> {
        if item.active && (item.quota_units > self.limit) {
            return vec![format!(
                "quota units {} exceeds {} while active",
                item.quota_units, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRegionPinTierRequiredCipherRule {
    pub message: String,
}

impl LenientRegionPinTierRule for LenientRegionPinTierRequiredCipherRule {
    fn check(&self, item: &LenientRegionPinTier) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRegionPinTierTimezoneLengthRule {
    pub max_length: i64,
}

impl LenientRegionPinTierRule for LenientRegionPinTierTimezoneLengthRule {
    fn check(&self, item: &LenientRegionPinTier) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRegionPinTierCodePrefixRule {
    pub prefix: String,
}

impl LenientRegionPinTierRule for LenientRegionPinTierCodePrefixRule {
    fn check(&self, item: &LenientRegionPinTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient region pin tier.
#[derive(Debug, Clone)]
pub struct LenientRegionPinTierValidator {
    pub rules: Vec<Rc<dyn LenientRegionPinTierRule>>,
}

impl LenientRegionPinTierValidator {
    pub fn validate(&self, item: &LenientRegionPinTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientRegionPinTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRegionPinTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_region_pin_tier_validator() -> LenientRegionPinTierValidator {
    LenientRegionPinTierValidator {
        rules: vec![
            Rc::new(LenientRegionPinTierActiveGuardRule { limit: 1000 }) as Rc<dyn LenientRegionPinTierRule>,
            Rc::new(LenientRegionPinTierRequiredCipherRule {
                message: "lenient region pin tier needs an entry".to_string(),
            }),
            Rc::new(LenientRegionPinTierTimezoneLengthRule { max_length: 1000 }),
            Rc::new(LenientRegionPinTierCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
