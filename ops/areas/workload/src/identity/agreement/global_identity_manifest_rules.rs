use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global identity manifest must pass. Check returns its problems, or none.
pub trait GlobalIdentityManifestRule: std::fmt::Debug {
    fn check(&self, item: &GlobalIdentityManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityManifestNameLengthRule {
    pub max_length: i64,
}

impl GlobalIdentityManifestRule for GlobalIdentityManifestNameLengthRule {
    fn check(&self, item: &GlobalIdentityManifest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityManifestMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalIdentityManifestRule for GlobalIdentityManifestMinimumRangeRule {
    fn check(&self, item: &GlobalIdentityManifest) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityManifestCipherLengthRule {
    pub max_length: i64,
}

impl GlobalIdentityManifestRule for GlobalIdentityManifestCipherLengthRule {
    fn check(&self, item: &GlobalIdentityManifest) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityManifestRequiredCipherRule {
    pub message: String,
}

impl GlobalIdentityManifestRule for GlobalIdentityManifestRequiredCipherRule {
    fn check(&self, item: &GlobalIdentityManifest) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityManifestRequiredTimezoneRule {
    pub message: String,
}

impl GlobalIdentityManifestRule for GlobalIdentityManifestRequiredTimezoneRule {
    fn check(&self, item: &GlobalIdentityManifest) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global identity manifest.
#[derive(Debug, Clone)]
pub struct GlobalIdentityManifestValidator {
    pub rules: Vec<Rc<dyn GlobalIdentityManifestRule>>,
}

impl GlobalIdentityManifestValidator {
    pub fn validate(&self, item: &GlobalIdentityManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalIdentityManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalIdentityManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_identity_manifest_validator() -> GlobalIdentityManifestValidator {
    GlobalIdentityManifestValidator {
        rules: vec![
            Rc::new(GlobalIdentityManifestNameLengthRule { max_length: 255 }) as Rc<dyn GlobalIdentityManifestRule>,
            Rc::new(GlobalIdentityManifestMinimumRangeRule { min: 8.8, max: 2770.5 }),
            Rc::new(GlobalIdentityManifestCipherLengthRule { max_length: 32 }),
            Rc::new(GlobalIdentityManifestRequiredCipherRule {
                message: "global identity manifest needs this field".to_string(),
            }),
            Rc::new(GlobalIdentityManifestRequiredTimezoneRule {
                message: "global identity manifest needs an entry".to_string(),
            }),
        ],
    }
}
