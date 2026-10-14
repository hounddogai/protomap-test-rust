use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global zone digest must pass. Check returns its problems, or none.
pub trait GlobalZoneDigestRule: std::fmt::Debug {
    fn check(&self, item: &GlobalZoneDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalZoneDigestLabelLengthRule {
    pub max_length: i64,
}

impl GlobalZoneDigestRule for GlobalZoneDigestLabelLengthRule {
    fn check(&self, item: &GlobalZoneDigest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalZoneDigestRequiredLabelRule {
    pub message: String,
}

impl GlobalZoneDigestRule for GlobalZoneDigestRequiredLabelRule {
    fn check(&self, item: &GlobalZoneDigest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalZoneDigestRequiredCodeRule {
    pub message: String,
}

impl GlobalZoneDigestRule for GlobalZoneDigestRequiredCodeRule {
    fn check(&self, item: &GlobalZoneDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalZoneDigestCodePrefixRule {
    pub prefix: String,
}

impl GlobalZoneDigestRule for GlobalZoneDigestCodePrefixRule {
    fn check(&self, item: &GlobalZoneDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global zone digest.
#[derive(Debug, Clone)]
pub struct GlobalZoneDigestValidator {
    pub rules: Vec<Rc<dyn GlobalZoneDigestRule>>,
}

impl GlobalZoneDigestValidator {
    pub fn validate(&self, item: &GlobalZoneDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalZoneDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalZoneDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_zone_digest_validator() -> GlobalZoneDigestValidator {
    GlobalZoneDigestValidator {
        rules: vec![
            Rc::new(GlobalZoneDigestLabelLengthRule { max_length: 120 }) as Rc<dyn GlobalZoneDigestRule>,
            Rc::new(GlobalZoneDigestRequiredLabelRule {
                message: "global zone digest needs this field".to_string(),
            }),
            Rc::new(GlobalZoneDigestRequiredCodeRule {
                message: "global zone digest needs a value".to_string(),
            }),
            Rc::new(GlobalZoneDigestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
