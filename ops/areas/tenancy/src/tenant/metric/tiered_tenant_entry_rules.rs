use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered tenant entry must pass. Check returns its problems, or none.
pub trait TieredTenantEntryRule: std::fmt::Debug {
    fn check(&self, item: &TieredTenantEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantEntryRequiredCodeRule {
    pub message: String,
}

impl TieredTenantEntryRule for TieredTenantEntryRequiredCodeRule {
    fn check(&self, item: &TieredTenantEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantEntryAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl TieredTenantEntryRule for TieredTenantEntryAllowedExternalRefRule {
    fn check(&self, item: &TieredTenantEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is below minimum", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantEntryRequiredCipherRule {
    pub message: String,
}

impl TieredTenantEntryRule for TieredTenantEntryRequiredCipherRule {
    fn check(&self, item: &TieredTenantEntry) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantEntryRequiredExternalRefRule {
    pub message: String,
}

impl TieredTenantEntryRule for TieredTenantEntryRequiredExternalRefRule {
    fn check(&self, item: &TieredTenantEntry) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered tenant entry.
#[derive(Debug, Clone)]
pub struct TieredTenantEntryValidator {
    pub rules: Vec<Rc<dyn TieredTenantEntryRule>>,
}

impl TieredTenantEntryValidator {
    pub fn validate(&self, item: &TieredTenantEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredTenantEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredTenantEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_tenant_entry_validator() -> TieredTenantEntryValidator {
    TieredTenantEntryValidator {
        rules: vec![
            Rc::new(TieredTenantEntryRequiredCodeRule {
                message: "tiered tenant entry needs this field".to_string(),
            }) as Rc<dyn TieredTenantEntryRule>,
            Rc::new(TieredTenantEntryAllowedExternalRefRule { allowed: Vec::new() }),
            Rc::new(TieredTenantEntryRequiredCipherRule {
                message: "tiered tenant entry needs this field".to_string(),
            }),
            Rc::new(TieredTenantEntryRequiredExternalRefRule {
                message: "tiered tenant entry needs an entry".to_string(),
            }),
        ],
    }
}
