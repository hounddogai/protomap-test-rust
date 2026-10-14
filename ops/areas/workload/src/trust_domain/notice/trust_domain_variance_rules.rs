use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain variance must pass. Check returns its problems, or none.
pub trait TrustDomainVarianceRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainVarianceRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainVarianceRule for TrustDomainVarianceRotationDaysRangeRule {
    fn check(&self, item: &TrustDomainVariance) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainVarianceRequiredRegionRule {
    pub message: String,
}

impl TrustDomainVarianceRule for TrustDomainVarianceRequiredRegionRule {
    fn check(&self, item: &TrustDomainVariance) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainVarianceStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainVarianceRule for TrustDomainVarianceStartsAtRangeRule {
    fn check(&self, item: &TrustDomainVariance) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainVarianceRequiredCodeRule {
    pub message: String,
}

impl TrustDomainVarianceRule for TrustDomainVarianceRequiredCodeRule {
    fn check(&self, item: &TrustDomainVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainVarianceRegionLengthRule {
    pub max_length: i64,
}

impl TrustDomainVarianceRule for TrustDomainVarianceRegionLengthRule {
    fn check(&self, item: &TrustDomainVariance) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain variance.
#[derive(Debug, Clone)]
pub struct TrustDomainVarianceValidator {
    pub rules: Vec<Rc<dyn TrustDomainVarianceRule>>,
}

impl TrustDomainVarianceValidator {
    pub fn validate(&self, item: &TrustDomainVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_variance_validator() -> TrustDomainVarianceValidator {
    TrustDomainVarianceValidator {
        rules: vec![
            Rc::new(TrustDomainVarianceRotationDaysRangeRule { min: 16, max: 3600 }) as Rc<dyn TrustDomainVarianceRule>,
            Rc::new(TrustDomainVarianceRequiredRegionRule {
                message: "trust domain variance needs an entry".to_string(),
            }),
            Rc::new(TrustDomainVarianceStartsAtRangeRule { min: 32, max: 255 }),
            Rc::new(TrustDomainVarianceRequiredCodeRule {
                message: "trust domain variance needs a value".to_string(),
            }),
            Rc::new(TrustDomainVarianceRegionLengthRule { max_length: 120 }),
        ],
    }
}
