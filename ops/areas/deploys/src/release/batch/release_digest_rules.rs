use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release digest must pass. Check returns its problems, or none.
pub trait ReleaseDigestRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseDigestArchivedGuardRule {
    pub limit: f64,
}

impl ReleaseDigestRule for ReleaseDigestArchivedGuardRule {
    fn check(&self, item: &ReleaseDigest) -> Vec<String> {
        if item.archived && (item.cost > self.limit) {
            return vec![format!("cost {} exceeds {} while archived", item.cost, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseDigestTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseDigestRule for ReleaseDigestTtlSecondsRangeRule {
    fn check(&self, item: &ReleaseDigest) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseDigestRequiredCodeRule {
    pub message: String,
}

impl ReleaseDigestRule for ReleaseDigestRequiredCodeRule {
    fn check(&self, item: &ReleaseDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseDigestRequiredLabelRule {
    pub message: String,
}

impl ReleaseDigestRule for ReleaseDigestRequiredLabelRule {
    fn check(&self, item: &ReleaseDigest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseDigestRequiredChecksumRule {
    pub message: String,
}

impl ReleaseDigestRule for ReleaseDigestRequiredChecksumRule {
    fn check(&self, item: &ReleaseDigest) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a release digest.
#[derive(Debug, Clone)]
pub struct ReleaseDigestValidator {
    pub rules: Vec<Rc<dyn ReleaseDigestRule>>,
}

impl ReleaseDigestValidator {
    pub fn validate(&self, item: &ReleaseDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReleaseDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_digest_validator() -> ReleaseDigestValidator {
    ReleaseDigestValidator {
        rules: vec![
            Rc::new(ReleaseDigestArchivedGuardRule { limit: 3.8 }) as Rc<dyn ReleaseDigestRule>,
            Rc::new(ReleaseDigestTtlSecondsRangeRule { min: 120, max: 255 }),
            Rc::new(ReleaseDigestRequiredCodeRule {
                message: "release digest needs a value".to_string(),
            }),
            Rc::new(ReleaseDigestRequiredLabelRule {
                message: "release digest needs a value".to_string(),
            }),
            Rc::new(ReleaseDigestRequiredChecksumRule {
                message: "release digest needs this field".to_string(),
            }),
        ],
    }
}
