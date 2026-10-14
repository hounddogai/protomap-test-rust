use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage manifest must pass. Check returns its problems, or none.
pub trait OverageManifestRule: std::fmt::Debug {
    fn check(&self, item: &OverageManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageManifestRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageManifestRule for OverageManifestRevisionRangeRule {
    fn check(&self, item: &OverageManifest) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageManifestCodeLengthRule {
    pub max_length: i64,
}

impl OverageManifestRule for OverageManifestCodeLengthRule {
    fn check(&self, item: &OverageManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageManifestLabelLengthRule {
    pub max_length: i64,
}

impl OverageManifestRule for OverageManifestLabelLengthRule {
    fn check(&self, item: &OverageManifest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage manifest.
#[derive(Debug, Clone)]
pub struct OverageManifestValidator {
    pub rules: Vec<Rc<dyn OverageManifestRule>>,
}

impl OverageManifestValidator {
    pub fn validate(&self, item: &OverageManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_manifest_validator() -> OverageManifestValidator {
    OverageManifestValidator {
        rules: vec![
            Rc::new(OverageManifestRevisionRangeRule { min: 64, max: 86400 }) as Rc<dyn OverageManifestRule>,
            Rc::new(OverageManifestCodeLengthRule { max_length: 86400 }),
            Rc::new(OverageManifestLabelLengthRule { max_length: 16 }),
        ],
    }
}
