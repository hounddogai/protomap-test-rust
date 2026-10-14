use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling grant target must pass. Check returns its problems, or none.
pub trait RollingGrantTargetRule: std::fmt::Debug {
    fn check(&self, item: &RollingGrantTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantTargetPrincipalLengthRule {
    pub max_length: i64,
}

impl RollingGrantTargetRule for RollingGrantTargetPrincipalLengthRule {
    fn check(&self, item: &RollingGrantTarget) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantTargetAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl RollingGrantTargetRule for RollingGrantTargetAllowedNamespaceRule {
    fn check(&self, item: &RollingGrantTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is too long", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantTargetCodePrefixRule {
    pub prefix: String,
}

impl RollingGrantTargetRule for RollingGrantTargetCodePrefixRule {
    fn check(&self, item: &RollingGrantTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantTargetRequiredChecksumRule {
    pub message: String,
}

impl RollingGrantTargetRule for RollingGrantTargetRequiredChecksumRule {
    fn check(&self, item: &RollingGrantTarget) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling grant target.
#[derive(Debug, Clone)]
pub struct RollingGrantTargetValidator {
    pub rules: Vec<Rc<dyn RollingGrantTargetRule>>,
}

impl RollingGrantTargetValidator {
    pub fn validate(&self, item: &RollingGrantTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingGrantTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingGrantTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_grant_target_validator() -> RollingGrantTargetValidator {
    RollingGrantTargetValidator {
        rules: vec![
            Rc::new(RollingGrantTargetPrincipalLengthRule { max_length: 80 }) as Rc<dyn RollingGrantTargetRule>,
            Rc::new(RollingGrantTargetAllowedNamespaceRule {
                allowed: vec!["primary".to_string(), "rolling".to_string(), "primary".to_string()],
            }),
            Rc::new(RollingGrantTargetCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RollingGrantTargetRequiredChecksumRule {
                message: "rolling grant target needs a value".to_string(),
            }),
        ],
    }
}
