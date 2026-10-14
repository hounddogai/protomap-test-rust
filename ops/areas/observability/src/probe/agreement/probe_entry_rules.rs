use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe entry must pass. Check returns its problems, or none.
pub trait ProbeEntryRule: std::fmt::Debug {
    fn check(&self, item: &ProbeEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeEntryKeyIdLengthRule {
    pub max_length: i64,
}

impl ProbeEntryRule for ProbeEntryKeyIdLengthRule {
    fn check(&self, item: &ProbeEntry) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeEntryRequiredTitleRule {
    pub message: String,
}

impl ProbeEntryRule for ProbeEntryRequiredTitleRule {
    fn check(&self, item: &ProbeEntry) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeEntryMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ProbeEntryRule for ProbeEntryMinimumRangeRule {
    fn check(&self, item: &ProbeEntry) -> Vec<String> {
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
pub struct ProbeEntryRequiredCodeRule {
    pub message: String,
}

impl ProbeEntryRule for ProbeEntryRequiredCodeRule {
    fn check(&self, item: &ProbeEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe entry.
#[derive(Debug, Clone)]
pub struct ProbeEntryValidator {
    pub rules: Vec<Rc<dyn ProbeEntryRule>>,
}

impl ProbeEntryValidator {
    pub fn validate(&self, item: &ProbeEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ProbeEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_entry_validator() -> ProbeEntryValidator {
    ProbeEntryValidator {
        rules: vec![
            Rc::new(ProbeEntryKeyIdLengthRule { max_length: 86400 }) as Rc<dyn ProbeEntryRule>,
            Rc::new(ProbeEntryRequiredTitleRule {
                message: "probe entry needs this field".to_string(),
            }),
            Rc::new(ProbeEntryMinimumRangeRule { min: 3.4, max: 192.5 }),
            Rc::new(ProbeEntryRequiredCodeRule {
                message: "probe entry needs an entry".to_string(),
            }),
        ],
    }
}
