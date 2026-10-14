use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling usage manifest must pass. Check returns its problems, or none.
pub trait RollingUsageManifestRule: std::fmt::Debug {
    fn check(&self, item: &RollingUsageManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingUsageManifestRequiredTitleRule {
    pub message: String,
}

impl RollingUsageManifestRule for RollingUsageManifestRequiredTitleRule {
    fn check(&self, item: &RollingUsageManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingUsageManifestRequiredSegmentRule {
    pub message: String,
}

impl RollingUsageManifestRule for RollingUsageManifestRequiredSegmentRule {
    fn check(&self, item: &RollingUsageManifest) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingUsageManifestWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingUsageManifestRule for RollingUsageManifestWeightRangeRule {
    fn check(&self, item: &RollingUsageManifest) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rolling usage manifest.
#[derive(Debug, Clone)]
pub struct RollingUsageManifestValidator {
    pub rules: Vec<Rc<dyn RollingUsageManifestRule>>,
}

impl RollingUsageManifestValidator {
    pub fn validate(&self, item: &RollingUsageManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingUsageManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingUsageManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_usage_manifest_validator() -> RollingUsageManifestValidator {
    RollingUsageManifestValidator {
        rules: vec![
            Rc::new(RollingUsageManifestRequiredTitleRule {
                message: "rolling usage manifest needs this field".to_string(),
            }) as Rc<dyn RollingUsageManifestRule>,
            Rc::new(RollingUsageManifestRequiredSegmentRule {
                message: "rolling usage manifest needs a value".to_string(),
            }),
            Rc::new(RollingUsageManifestWeightRangeRule { min: 9.4, max: 2335.0 }),
        ],
    }
}
