use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation manifest must pass. Check returns its problems, or none.
pub trait FederationManifestRule: std::fmt::Debug {
    fn check(&self, item: &FederationManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationManifestAudienceLengthRule {
    pub max_length: i64,
}

impl FederationManifestRule for FederationManifestAudienceLengthRule {
    fn check(&self, item: &FederationManifest) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationManifestRequiredCodeRule {
    pub message: String,
}

impl FederationManifestRule for FederationManifestRequiredCodeRule {
    fn check(&self, item: &FederationManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationManifestTitleLengthRule {
    pub max_length: i64,
}

impl FederationManifestRule for FederationManifestTitleLengthRule {
    fn check(&self, item: &FederationManifest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationManifestCipherLengthRule {
    pub max_length: i64,
}

impl FederationManifestRule for FederationManifestCipherLengthRule {
    fn check(&self, item: &FederationManifest) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationManifestRequiredTitleRule {
    pub message: String,
}

impl FederationManifestRule for FederationManifestRequiredTitleRule {
    fn check(&self, item: &FederationManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation manifest.
#[derive(Debug, Clone)]
pub struct FederationManifestValidator {
    pub rules: Vec<Rc<dyn FederationManifestRule>>,
}

impl FederationManifestValidator {
    pub fn validate(&self, item: &FederationManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_manifest_validator() -> FederationManifestValidator {
    FederationManifestValidator {
        rules: vec![
            Rc::new(FederationManifestAudienceLengthRule { max_length: 86400 }) as Rc<dyn FederationManifestRule>,
            Rc::new(FederationManifestRequiredCodeRule {
                message: "federation manifest needs this field".to_string(),
            }),
            Rc::new(FederationManifestTitleLengthRule { max_length: 1000 }),
            Rc::new(FederationManifestCipherLengthRule { max_length: 255 }),
            Rc::new(FederationManifestRequiredTitleRule {
                message: "federation manifest needs this field".to_string(),
            }),
        ],
    }
}
