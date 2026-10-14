use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a zone profile must pass. Check returns its problems, or none.
pub trait ZoneProfileRule: std::fmt::Debug {
    fn check(&self, item: &ZoneProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneProfileActiveGuardRule {
    pub limit: i64,
}

impl ZoneProfileRule for ZoneProfileActiveGuardRule {
    fn check(&self, item: &ZoneProfile) -> Vec<String> {
        if item.active && (item.batch_size > self.limit) {
            return vec![format!(
                "batch size {} exceeds {} while active",
                item.batch_size, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneProfileCodeLengthRule {
    pub max_length: i64,
}

impl ZoneProfileRule for ZoneProfileCodeLengthRule {
    fn check(&self, item: &ZoneProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneProfilePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneProfileRule for ZoneProfilePriorityRangeRule {
    fn check(&self, item: &ZoneProfile) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneProfileNameLengthRule {
    pub max_length: i64,
}

impl ZoneProfileRule for ZoneProfileNameLengthRule {
    fn check(&self, item: &ZoneProfile) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneProfileRequiredExternalRefRule {
    pub message: String,
}

impl ZoneProfileRule for ZoneProfileRequiredExternalRefRule {
    fn check(&self, item: &ZoneProfile) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneProfileCodePrefixRule {
    pub prefix: String,
}

impl ZoneProfileRule for ZoneProfileCodePrefixRule {
    fn check(&self, item: &ZoneProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone profile.
#[derive(Debug, Clone)]
pub struct ZoneProfileValidator {
    pub rules: Vec<Rc<dyn ZoneProfileRule>>,
}

impl ZoneProfileValidator {
    pub fn validate(&self, item: &ZoneProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ZoneProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_profile_validator() -> ZoneProfileValidator {
    ZoneProfileValidator {
        rules: vec![
            Rc::new(ZoneProfileActiveGuardRule { limit: 120 }) as Rc<dyn ZoneProfileRule>,
            Rc::new(ZoneProfileCodeLengthRule { max_length: 86400 }),
            Rc::new(ZoneProfilePriorityRangeRule { min: 16, max: 3600 }),
            Rc::new(ZoneProfileNameLengthRule { max_length: 64 }),
            Rc::new(ZoneProfileRequiredExternalRefRule {
                message: "zone profile needs this field".to_string(),
            }),
            Rc::new(ZoneProfileCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
