use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary postmortem rule must pass. Check returns its problems, or none.
pub trait PrimaryPostmortemRuleRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryPostmortemRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPostmortemRuleRequiredFingerprintRule {
    pub message: String,
}

impl PrimaryPostmortemRuleRule for PrimaryPostmortemRuleRequiredFingerprintRule {
    fn check(&self, item: &PrimaryPostmortemRule) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPostmortemRuleRequiredExternalRefRule {
    pub message: String,
}

impl PrimaryPostmortemRuleRule for PrimaryPostmortemRuleRequiredExternalRefRule {
    fn check(&self, item: &PrimaryPostmortemRule) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPostmortemRuleCodePrefixRule {
    pub prefix: String,
}

impl PrimaryPostmortemRuleRule for PrimaryPostmortemRuleCodePrefixRule {
    fn check(&self, item: &PrimaryPostmortemRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPostmortemRuleAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl PrimaryPostmortemRuleRule for PrimaryPostmortemRuleAllowedDescriptionRule {
    fn check(&self, item: &PrimaryPostmortemRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is stale", item.description)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPostmortemRuleExternalRefLengthRule {
    pub max_length: i64,
}

impl PrimaryPostmortemRuleRule for PrimaryPostmortemRuleExternalRefLengthRule {
    fn check(&self, item: &PrimaryPostmortemRule) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPostmortemRuleFingerprintLengthRule {
    pub max_length: i64,
}

impl PrimaryPostmortemRuleRule for PrimaryPostmortemRuleFingerprintLengthRule {
    fn check(&self, item: &PrimaryPostmortemRule) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary postmortem rule.
#[derive(Debug, Clone)]
pub struct PrimaryPostmortemRuleValidator {
    pub rules: Vec<Rc<dyn PrimaryPostmortemRuleRule>>,
}

impl PrimaryPostmortemRuleValidator {
    pub fn validate(&self, item: &PrimaryPostmortemRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryPostmortemRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryPostmortemRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_postmortem_rule_validator() -> PrimaryPostmortemRuleValidator {
    PrimaryPostmortemRuleValidator {
        rules: vec![
            Rc::new(PrimaryPostmortemRuleRequiredFingerprintRule {
                message: "primary postmortem rule needs this field".to_string(),
            }) as Rc<dyn PrimaryPostmortemRuleRule>,
            Rc::new(PrimaryPostmortemRuleRequiredExternalRefRule {
                message: "primary postmortem rule needs an entry".to_string(),
            }),
            Rc::new(PrimaryPostmortemRuleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(PrimaryPostmortemRuleAllowedDescriptionRule { allowed: Vec::new() }),
            Rc::new(PrimaryPostmortemRuleExternalRefLengthRule { max_length: 80 }),
            Rc::new(PrimaryPostmortemRuleFingerprintLengthRule { max_length: 32 }),
        ],
    }
}
