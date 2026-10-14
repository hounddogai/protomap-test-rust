use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation tier must pass. Check returns its problems, or none.
pub trait DelegationTierRule: std::fmt::Debug {
    fn check(&self, item: &DelegationTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTierCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationTierRule for DelegationTierCreatedAtRangeRule {
    fn check(&self, item: &DelegationTier) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTierSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationTierRule for DelegationTierSequenceRangeRule {
    fn check(&self, item: &DelegationTier) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTierCodeLengthRule {
    pub max_length: i64,
}

impl DelegationTierRule for DelegationTierCodeLengthRule {
    fn check(&self, item: &DelegationTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTierRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DelegationTierRule for DelegationTierRatioRangeRule {
    fn check(&self, item: &DelegationTier) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTierNameLengthRule {
    pub max_length: i64,
}

impl DelegationTierRule for DelegationTierNameLengthRule {
    fn check(&self, item: &DelegationTier) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTierCategoryLengthRule {
    pub max_length: i64,
}

impl DelegationTierRule for DelegationTierCategoryLengthRule {
    fn check(&self, item: &DelegationTier) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation tier.
#[derive(Debug, Clone)]
pub struct DelegationTierValidator {
    pub rules: Vec<Rc<dyn DelegationTierRule>>,
}

impl DelegationTierValidator {
    pub fn validate(&self, item: &DelegationTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DelegationTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_tier_validator() -> DelegationTierValidator {
    DelegationTierValidator {
        rules: vec![
            Rc::new(DelegationTierCreatedAtRangeRule { min: 120, max: 3600 }) as Rc<dyn DelegationTierRule>,
            Rc::new(DelegationTierSequenceRangeRule { min: 32, max: 120 }),
            Rc::new(DelegationTierCodeLengthRule { max_length: 32 }),
            Rc::new(DelegationTierRatioRangeRule { min: 6.4, max: 572.3 }),
            Rc::new(DelegationTierNameLengthRule { max_length: 32 }),
            Rc::new(DelegationTierCategoryLengthRule { max_length: 86400 }),
        ],
    }
}
