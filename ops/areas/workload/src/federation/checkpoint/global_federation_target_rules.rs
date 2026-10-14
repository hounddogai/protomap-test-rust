use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global federation target must pass. Check returns its problems, or none.
pub trait GlobalFederationTargetRule: std::fmt::Debug {
    fn check(&self, item: &GlobalFederationTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFederationTargetNameLengthRule {
    pub max_length: i64,
}

impl GlobalFederationTargetRule for GlobalFederationTargetNameLengthRule {
    fn check(&self, item: &GlobalFederationTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFederationTargetRequiredUnitRule {
    pub message: String,
}

impl GlobalFederationTargetRule for GlobalFederationTargetRequiredUnitRule {
    fn check(&self, item: &GlobalFederationTarget) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFederationTargetAllowedNameRule {
    pub allowed: Vec<String>,
}

impl GlobalFederationTargetRule for GlobalFederationTargetAllowedNameRule {
    fn check(&self, item: &GlobalFederationTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global federation target.
#[derive(Debug, Clone)]
pub struct GlobalFederationTargetValidator {
    pub rules: Vec<Rc<dyn GlobalFederationTargetRule>>,
}

impl GlobalFederationTargetValidator {
    pub fn validate(&self, item: &GlobalFederationTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalFederationTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalFederationTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_federation_target_validator() -> GlobalFederationTargetValidator {
    GlobalFederationTargetValidator {
        rules: vec![
            Rc::new(GlobalFederationTargetNameLengthRule { max_length: 64 }) as Rc<dyn GlobalFederationTargetRule>,
            Rc::new(GlobalFederationTargetRequiredUnitRule {
                message: "global federation target needs this field".to_string(),
            }),
            Rc::new(GlobalFederationTargetAllowedNameRule { allowed: Vec::new() }),
        ],
    }
}
