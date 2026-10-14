use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global silence target must pass. Check returns its problems, or none.
pub trait GlobalSilenceTargetRule: std::fmt::Debug {
    fn check(&self, item: &GlobalSilenceTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceTargetAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl GlobalSilenceTargetRule for GlobalSilenceTargetAllowedCodeRule {
    fn check(&self, item: &GlobalSilenceTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceTargetRequiredTenantIdRule {
    pub message: String,
}

impl GlobalSilenceTargetRule for GlobalSilenceTargetRequiredTenantIdRule {
    fn check(&self, item: &GlobalSilenceTarget) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceTargetCodePrefixRule {
    pub prefix: String,
}

impl GlobalSilenceTargetRule for GlobalSilenceTargetCodePrefixRule {
    fn check(&self, item: &GlobalSilenceTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceTargetCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalSilenceTargetRule for GlobalSilenceTargetCapacityRangeRule {
    fn check(&self, item: &GlobalSilenceTarget) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a global silence target.
#[derive(Debug, Clone)]
pub struct GlobalSilenceTargetValidator {
    pub rules: Vec<Rc<dyn GlobalSilenceTargetRule>>,
}

impl GlobalSilenceTargetValidator {
    pub fn validate(&self, item: &GlobalSilenceTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalSilenceTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalSilenceTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_silence_target_validator() -> GlobalSilenceTargetValidator {
    GlobalSilenceTargetValidator {
        rules: vec![
            Rc::new(GlobalSilenceTargetAllowedCodeRule { allowed: Vec::new() }) as Rc<dyn GlobalSilenceTargetRule>,
            Rc::new(GlobalSilenceTargetRequiredTenantIdRule {
                message: "global silence target needs a value".to_string(),
            }),
            Rc::new(GlobalSilenceTargetCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(GlobalSilenceTargetCapacityRangeRule { min: 80, max: 255 }),
        ],
    }
}
