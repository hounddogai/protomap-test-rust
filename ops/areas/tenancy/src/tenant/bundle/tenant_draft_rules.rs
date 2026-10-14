use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant draft must pass. Check returns its problems, or none.
pub trait TenantDraftRule: std::fmt::Debug {
    fn check(&self, item: &TenantDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantDraftRequiredNameRule {
    pub message: String,
}

impl TenantDraftRule for TenantDraftRequiredNameRule {
    fn check(&self, item: &TenantDraft) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantDraftBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TenantDraftRule for TenantDraftBurstRangeRule {
    fn check(&self, item: &TenantDraft) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantDraftCodePrefixRule {
    pub prefix: String,
}

impl TenantDraftRule for TenantDraftCodePrefixRule {
    fn check(&self, item: &TenantDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantDraftCodeLengthRule {
    pub max_length: i64,
}

impl TenantDraftRule for TenantDraftCodeLengthRule {
    fn check(&self, item: &TenantDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantDraftRegionLengthRule {
    pub max_length: i64,
}

impl TenantDraftRule for TenantDraftRegionLengthRule {
    fn check(&self, item: &TenantDraft) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantDraftRequiredRegionRule {
    pub message: String,
}

impl TenantDraftRule for TenantDraftRequiredRegionRule {
    fn check(&self, item: &TenantDraft) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant draft.
#[derive(Debug, Clone)]
pub struct TenantDraftValidator {
    pub rules: Vec<Rc<dyn TenantDraftRule>>,
}

impl TenantDraftValidator {
    pub fn validate(&self, item: &TenantDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_draft_validator() -> TenantDraftValidator {
    TenantDraftValidator {
        rules: vec![
            Rc::new(TenantDraftRequiredNameRule {
                message: "tenant draft needs an entry".to_string(),
            }) as Rc<dyn TenantDraftRule>,
            Rc::new(TenantDraftBurstRangeRule { min: 16, max: 120 }),
            Rc::new(TenantDraftCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(TenantDraftCodeLengthRule { max_length: 80 }),
            Rc::new(TenantDraftRegionLengthRule { max_length: 32 }),
            Rc::new(TenantDraftRequiredRegionRule {
                message: "tenant draft needs an entry".to_string(),
            }),
        ],
    }
}
