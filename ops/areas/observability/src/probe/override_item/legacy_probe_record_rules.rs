use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy probe record must pass. Check returns its problems, or none.
pub trait LegacyProbeRecordRule: std::fmt::Debug {
    fn check(&self, item: &LegacyProbeRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeRecordCodePrefixRule {
    pub prefix: String,
}

impl LegacyProbeRecordRule for LegacyProbeRecordCodePrefixRule {
    fn check(&self, item: &LegacyProbeRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeRecordRequiredNameRule {
    pub message: String,
}

impl LegacyProbeRecordRule for LegacyProbeRecordRequiredNameRule {
    fn check(&self, item: &LegacyProbeRecord) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeRecordLocaleLengthRule {
    pub max_length: i64,
}

impl LegacyProbeRecordRule for LegacyProbeRecordLocaleLengthRule {
    fn check(&self, item: &LegacyProbeRecord) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeRecordRequiredLocaleRule {
    pub message: String,
}

impl LegacyProbeRecordRule for LegacyProbeRecordRequiredLocaleRule {
    fn check(&self, item: &LegacyProbeRecord) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy probe record.
#[derive(Debug, Clone)]
pub struct LegacyProbeRecordValidator {
    pub rules: Vec<Rc<dyn LegacyProbeRecordRule>>,
}

impl LegacyProbeRecordValidator {
    pub fn validate(&self, item: &LegacyProbeRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyProbeRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyProbeRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_probe_record_validator() -> LegacyProbeRecordValidator {
    LegacyProbeRecordValidator {
        rules: vec![
            Rc::new(LegacyProbeRecordCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn LegacyProbeRecordRule>,
            Rc::new(LegacyProbeRecordRequiredNameRule {
                message: "legacy probe record needs an entry".to_string(),
            }),
            Rc::new(LegacyProbeRecordLocaleLengthRule { max_length: 3600 }),
            Rc::new(LegacyProbeRecordRequiredLocaleRule {
                message: "legacy probe record needs this field".to_string(),
            }),
        ],
    }
}
