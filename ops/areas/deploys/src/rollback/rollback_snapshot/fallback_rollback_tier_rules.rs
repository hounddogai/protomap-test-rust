use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback rollback tier must pass. Check returns its problems, or none.
pub trait FallbackRollbackTierRule: std::fmt::Debug {
    fn check(&self, item: &FallbackRollbackTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRollbackTierRequiredCodeRule {
    pub message: String,
}

impl FallbackRollbackTierRule for FallbackRollbackTierRequiredCodeRule {
    fn check(&self, item: &FallbackRollbackTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRollbackTierQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackRollbackTierRule for FallbackRollbackTierQuotaUnitsRangeRule {
    fn check(&self, item: &FallbackRollbackTier) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRollbackTierSegmentLengthRule {
    pub max_length: i64,
}

impl FallbackRollbackTierRule for FallbackRollbackTierSegmentLengthRule {
    fn check(&self, item: &FallbackRollbackTier) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRollbackTierRequiredTitleRule {
    pub message: String,
}

impl FallbackRollbackTierRule for FallbackRollbackTierRequiredTitleRule {
    fn check(&self, item: &FallbackRollbackTier) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRollbackTierRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackRollbackTierRule for FallbackRollbackTierRatioRangeRule {
    fn check(&self, item: &FallbackRollbackTier) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback rollback tier.
#[derive(Debug, Clone)]
pub struct FallbackRollbackTierValidator {
    pub rules: Vec<Rc<dyn FallbackRollbackTierRule>>,
}

impl FallbackRollbackTierValidator {
    pub fn validate(&self, item: &FallbackRollbackTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackRollbackTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackRollbackTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_rollback_tier_validator() -> FallbackRollbackTierValidator {
    FallbackRollbackTierValidator {
        rules: vec![
            Rc::new(FallbackRollbackTierRequiredCodeRule {
                message: "fallback rollback tier needs this field".to_string(),
            }) as Rc<dyn FallbackRollbackTierRule>,
            Rc::new(FallbackRollbackTierQuotaUnitsRangeRule { min: 120, max: 86400 }),
            Rc::new(FallbackRollbackTierSegmentLengthRule { max_length: 1000 }),
            Rc::new(FallbackRollbackTierRequiredTitleRule {
                message: "fallback rollback tier needs this field".to_string(),
            }),
            Rc::new(FallbackRollbackTierRatioRangeRule { min: 2.0, max: 3886.8 }),
        ],
    }
}
