use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a evidence override must pass. Check returns its problems, or none.
pub trait EvidenceOverrideRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceOverrideCodePrefixRule {
    pub prefix: String,
}

impl EvidenceOverrideRule for EvidenceOverrideCodePrefixRule {
    fn check(&self, item: &EvidenceOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceOverrideAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl EvidenceOverrideRule for EvidenceOverrideAllowedNamespaceRule {
    fn check(&self, item: &EvidenceOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is expired", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceOverrideRequiredNameRule {
    pub message: String,
}

impl EvidenceOverrideRule for EvidenceOverrideRequiredNameRule {
    fn check(&self, item: &EvidenceOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence override.
#[derive(Debug, Clone)]
pub struct EvidenceOverrideValidator {
    pub rules: Vec<Rc<dyn EvidenceOverrideRule>>,
}

impl EvidenceOverrideValidator {
    pub fn validate(&self, item: &EvidenceOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_override_validator() -> EvidenceOverrideValidator {
    EvidenceOverrideValidator {
        rules: vec![
            Rc::new(EvidenceOverrideCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn EvidenceOverrideRule>,
            Rc::new(EvidenceOverrideAllowedNamespaceRule { allowed: Vec::new() }),
            Rc::new(EvidenceOverrideRequiredNameRule {
                message: "evidence override needs a value".to_string(),
            }),
        ],
    }
}
