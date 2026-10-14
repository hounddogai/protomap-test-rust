use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation manifest must pass. Check returns its problems, or none.
pub trait DelegationManifestRule: std::fmt::Debug {
    fn check(&self, item: &DelegationManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationManifestRequiredCodeRule {
    pub message: String,
}

impl DelegationManifestRule for DelegationManifestRequiredCodeRule {
    fn check(&self, item: &DelegationManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationManifestPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationManifestRule for DelegationManifestPortRangeRule {
    fn check(&self, item: &DelegationManifest) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationManifestSummaryLengthRule {
    pub max_length: i64,
}

impl DelegationManifestRule for DelegationManifestSummaryLengthRule {
    fn check(&self, item: &DelegationManifest) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation manifest.
#[derive(Debug, Clone)]
pub struct DelegationManifestValidator {
    pub rules: Vec<Rc<dyn DelegationManifestRule>>,
}

impl DelegationManifestValidator {
    pub fn validate(&self, item: &DelegationManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_manifest_validator() -> DelegationManifestValidator {
    DelegationManifestValidator {
        rules: vec![
            Rc::new(DelegationManifestRequiredCodeRule {
                message: "delegation manifest needs a value".to_string(),
            }) as Rc<dyn DelegationManifestRule>,
            Rc::new(DelegationManifestPortRangeRule { min: 80, max: 255 }),
            Rc::new(DelegationManifestSummaryLengthRule { max_length: 120 }),
        ],
    }
}
