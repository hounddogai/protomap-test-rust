use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary reservation manifest must pass. Check returns its problems, or none.
pub trait PrimaryReservationManifestRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryReservationManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationManifestStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryReservationManifestRule for PrimaryReservationManifestStartsAtRangeRule {
    fn check(&self, item: &PrimaryReservationManifest) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationManifestRequiredTitleRule {
    pub message: String,
}

impl PrimaryReservationManifestRule for PrimaryReservationManifestRequiredTitleRule {
    fn check(&self, item: &PrimaryReservationManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationManifestThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryReservationManifestRule for PrimaryReservationManifestThresholdRangeRule {
    fn check(&self, item: &PrimaryReservationManifest) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationManifestIssuerLengthRule {
    pub max_length: i64,
}

impl PrimaryReservationManifestRule for PrimaryReservationManifestIssuerLengthRule {
    fn check(&self, item: &PrimaryReservationManifest) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationManifestTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryReservationManifestRule for PrimaryReservationManifestTitleLengthRule {
    fn check(&self, item: &PrimaryReservationManifest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary reservation manifest.
#[derive(Debug, Clone)]
pub struct PrimaryReservationManifestValidator {
    pub rules: Vec<Rc<dyn PrimaryReservationManifestRule>>,
}

impl PrimaryReservationManifestValidator {
    pub fn validate(&self, item: &PrimaryReservationManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryReservationManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryReservationManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_reservation_manifest_validator() -> PrimaryReservationManifestValidator {
    PrimaryReservationManifestValidator {
        rules: vec![
            Rc::new(PrimaryReservationManifestStartsAtRangeRule { min: 32, max: 120 })
                as Rc<dyn PrimaryReservationManifestRule>,
            Rc::new(PrimaryReservationManifestRequiredTitleRule {
                message: "primary reservation manifest needs a value".to_string(),
            }),
            Rc::new(PrimaryReservationManifestThresholdRangeRule { min: 1.8, max: 3982.9 }),
            Rc::new(PrimaryReservationManifestIssuerLengthRule { max_length: 86400 }),
            Rc::new(PrimaryReservationManifestTitleLengthRule { max_length: 64 }),
        ],
    }
}
