use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence manifest must pass. Check returns its problems, or none.
pub trait EvidenceManifestRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceManifestCodeLengthRule {
    pub max_length: i64,
}

impl EvidenceManifestRule for EvidenceManifestCodeLengthRule {
    fn check(&self, item: &EvidenceManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceManifestContactEmailLengthRule {
    pub max_length: i64,
}

impl EvidenceManifestRule for EvidenceManifestContactEmailLengthRule {
    fn check(&self, item: &EvidenceManifest) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceManifestRequiredCodeRule {
    pub message: String,
}

impl EvidenceManifestRule for EvidenceManifestRequiredCodeRule {
    fn check(&self, item: &EvidenceManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence manifest.
#[derive(Debug, Clone)]
pub struct EvidenceManifestValidator {
    pub rules: Vec<Rc<dyn EvidenceManifestRule>>,
}

impl EvidenceManifestValidator {
    pub fn validate(&self, item: &EvidenceManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EvidenceManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_manifest_validator() -> EvidenceManifestValidator {
    EvidenceManifestValidator {
        rules: vec![
            Rc::new(EvidenceManifestCodeLengthRule { max_length: 32 }) as Rc<dyn EvidenceManifestRule>,
            Rc::new(EvidenceManifestContactEmailLengthRule { max_length: 32 }),
            Rc::new(EvidenceManifestRequiredCodeRule {
                message: "evidence manifest needs an entry".to_string(),
            }),
        ],
    }
}
