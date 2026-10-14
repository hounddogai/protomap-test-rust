use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy evidence exception must pass. Check returns its problems, or none.
pub trait LegacyEvidenceExceptionRule: std::fmt::Debug {
    fn check(&self, item: &LegacyEvidenceException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceExceptionRequiredTenantIdRule {
    pub message: String,
}

impl LegacyEvidenceExceptionRule for LegacyEvidenceExceptionRequiredTenantIdRule {
    fn check(&self, item: &LegacyEvidenceException) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceExceptionCodeLengthRule {
    pub max_length: i64,
}

impl LegacyEvidenceExceptionRule for LegacyEvidenceExceptionCodeLengthRule {
    fn check(&self, item: &LegacyEvidenceException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceExceptionRequiredRegionRule {
    pub message: String,
}

impl LegacyEvidenceExceptionRule for LegacyEvidenceExceptionRequiredRegionRule {
    fn check(&self, item: &LegacyEvidenceException) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceExceptionRequiredNameRule {
    pub message: String,
}

impl LegacyEvidenceExceptionRule for LegacyEvidenceExceptionRequiredNameRule {
    fn check(&self, item: &LegacyEvidenceException) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceExceptionCodePrefixRule {
    pub prefix: String,
}

impl LegacyEvidenceExceptionRule for LegacyEvidenceExceptionCodePrefixRule {
    fn check(&self, item: &LegacyEvidenceException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy evidence exception.
#[derive(Debug, Clone)]
pub struct LegacyEvidenceExceptionValidator {
    pub rules: Vec<Rc<dyn LegacyEvidenceExceptionRule>>,
}

impl LegacyEvidenceExceptionValidator {
    pub fn validate(&self, item: &LegacyEvidenceException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyEvidenceException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyEvidenceException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_evidence_exception_validator() -> LegacyEvidenceExceptionValidator {
    LegacyEvidenceExceptionValidator {
        rules: vec![
            Rc::new(LegacyEvidenceExceptionRequiredTenantIdRule {
                message: "legacy evidence exception needs this field".to_string(),
            }) as Rc<dyn LegacyEvidenceExceptionRule>,
            Rc::new(LegacyEvidenceExceptionCodeLengthRule { max_length: 32 }),
            Rc::new(LegacyEvidenceExceptionRequiredRegionRule {
                message: "legacy evidence exception needs an entry".to_string(),
            }),
            Rc::new(LegacyEvidenceExceptionRequiredNameRule {
                message: "legacy evidence exception needs an entry".to_string(),
            }),
            Rc::new(LegacyEvidenceExceptionCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
