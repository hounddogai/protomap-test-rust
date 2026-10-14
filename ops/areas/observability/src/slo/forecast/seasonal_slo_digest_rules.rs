use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal slo digest must pass. Check returns its problems, or none.
pub trait SeasonalSloDigestRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalSloDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloDigestQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalSloDigestRule for SeasonalSloDigestQuotaUnitsRangeRule {
    fn check(&self, item: &SeasonalSloDigest) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloDigestCountryLengthRule {
    pub max_length: i64,
}

impl SeasonalSloDigestRule for SeasonalSloDigestCountryLengthRule {
    fn check(&self, item: &SeasonalSloDigest) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloDigestRequiredFingerprintRule {
    pub message: String,
}

impl SeasonalSloDigestRule for SeasonalSloDigestRequiredFingerprintRule {
    fn check(&self, item: &SeasonalSloDigest) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloDigestRequiredCountryRule {
    pub message: String,
}

impl SeasonalSloDigestRule for SeasonalSloDigestRequiredCountryRule {
    fn check(&self, item: &SeasonalSloDigest) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloDigestCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalSloDigestRule for SeasonalSloDigestCreatedAtRangeRule {
    fn check(&self, item: &SeasonalSloDigest) -> Vec<String> {
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

/// Runs every rule of a seasonal slo digest.
#[derive(Debug, Clone)]
pub struct SeasonalSloDigestValidator {
    pub rules: Vec<Rc<dyn SeasonalSloDigestRule>>,
}

impl SeasonalSloDigestValidator {
    pub fn validate(&self, item: &SeasonalSloDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalSloDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalSloDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_slo_digest_validator() -> SeasonalSloDigestValidator {
    SeasonalSloDigestValidator {
        rules: vec![
            Rc::new(SeasonalSloDigestQuotaUnitsRangeRule { min: 255, max: 255 }) as Rc<dyn SeasonalSloDigestRule>,
            Rc::new(SeasonalSloDigestCountryLengthRule { max_length: 1000 }),
            Rc::new(SeasonalSloDigestRequiredFingerprintRule {
                message: "seasonal slo digest needs an entry".to_string(),
            }),
            Rc::new(SeasonalSloDigestRequiredCountryRule {
                message: "seasonal slo digest needs this field".to_string(),
            }),
            Rc::new(SeasonalSloDigestCreatedAtRangeRule { min: 80, max: 255 }),
        ],
    }
}
