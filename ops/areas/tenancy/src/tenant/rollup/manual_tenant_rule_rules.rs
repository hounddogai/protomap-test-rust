use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual tenant rule must pass. Check returns its problems, or none.
pub trait ManualTenantRuleRule: std::fmt::Debug {
    fn check(&self, item: &ManualTenantRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantRuleAllowedNameRule {
    pub allowed: Vec<String>,
}

impl ManualTenantRuleRule for ManualTenantRuleAllowedNameRule {
    fn check(&self, item: &ManualTenantRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantRuleRequiredNameRule {
    pub message: String,
}

impl ManualTenantRuleRule for ManualTenantRuleRequiredNameRule {
    fn check(&self, item: &ManualTenantRule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantRuleRequiredCipherRule {
    pub message: String,
}

impl ManualTenantRuleRule for ManualTenantRuleRequiredCipherRule {
    fn check(&self, item: &ManualTenantRule) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantRuleRequiredCodeRule {
    pub message: String,
}

impl ManualTenantRuleRule for ManualTenantRuleRequiredCodeRule {
    fn check(&self, item: &ManualTenantRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual tenant rule.
#[derive(Debug, Clone)]
pub struct ManualTenantRuleValidator {
    pub rules: Vec<Rc<dyn ManualTenantRuleRule>>,
}

impl ManualTenantRuleValidator {
    pub fn validate(&self, item: &ManualTenantRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualTenantRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualTenantRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_tenant_rule_validator() -> ManualTenantRuleValidator {
    ManualTenantRuleValidator {
        rules: vec![
            Rc::new(ManualTenantRuleAllowedNameRule {
                allowed: vec!["default".to_string(), "rolling".to_string()],
            }) as Rc<dyn ManualTenantRuleRule>,
            Rc::new(ManualTenantRuleRequiredNameRule {
                message: "manual tenant rule needs an entry".to_string(),
            }),
            Rc::new(ManualTenantRuleRequiredCipherRule {
                message: "manual tenant rule needs this field".to_string(),
            }),
            Rc::new(ManualTenantRuleRequiredCodeRule {
                message: "manual tenant rule needs a value".to_string(),
            }),
        ],
    }
}
