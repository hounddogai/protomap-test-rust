use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier manifest must pass. Check returns its problems, or none.
pub trait VerifierManifestRule: std::fmt::Debug {
    fn check(&self, item: &VerifierManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierManifestBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierManifestRule for VerifierManifestBatchSizeRangeRule {
    fn check(&self, item: &VerifierManifest) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierManifestTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl VerifierManifestRule for VerifierManifestTagsLimitRule {
    fn check(&self, item: &VerifierManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierManifestRequiredLabelRule {
    pub message: String,
}

impl VerifierManifestRule for VerifierManifestRequiredLabelRule {
    fn check(&self, item: &VerifierManifest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier manifest.
#[derive(Debug, Clone)]
pub struct VerifierManifestValidator {
    pub rules: Vec<Rc<dyn VerifierManifestRule>>,
}

impl VerifierManifestValidator {
    pub fn validate(&self, item: &VerifierManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &VerifierManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_manifest_validator() -> VerifierManifestValidator {
    VerifierManifestValidator {
        rules: vec![
            Rc::new(VerifierManifestBatchSizeRangeRule { min: 255, max: 1000 }) as Rc<dyn VerifierManifestRule>,
            Rc::new(VerifierManifestTagsLimitRule {
                max_count: 86400,
                banned: vec!["global".to_string(), "fallback".to_string()],
            }),
            Rc::new(VerifierManifestRequiredLabelRule {
                message: "verifier manifest needs a value".to_string(),
            }),
        ],
    }
}
