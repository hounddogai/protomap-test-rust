use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy data export allocation must pass. Check returns its problems, or none.
pub trait LegacyDataExportAllocationRule: std::fmt::Debug {
    fn check(&self, item: &LegacyDataExportAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportAllocationSourceLengthRule {
    pub max_length: i64,
}

impl LegacyDataExportAllocationRule for LegacyDataExportAllocationSourceLengthRule {
    fn check(&self, item: &LegacyDataExportAllocation) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportAllocationPhoneLengthRule {
    pub max_length: i64,
}

impl LegacyDataExportAllocationRule for LegacyDataExportAllocationPhoneLengthRule {
    fn check(&self, item: &LegacyDataExportAllocation) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportAllocationCodePrefixRule {
    pub prefix: String,
}

impl LegacyDataExportAllocationRule for LegacyDataExportAllocationCodePrefixRule {
    fn check(&self, item: &LegacyDataExportAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportAllocationBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyDataExportAllocationRule for LegacyDataExportAllocationBurstRangeRule {
    fn check(&self, item: &LegacyDataExportAllocation) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportAllocationCodeLengthRule {
    pub max_length: i64,
}

impl LegacyDataExportAllocationRule for LegacyDataExportAllocationCodeLengthRule {
    fn check(&self, item: &LegacyDataExportAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportAllocationDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyDataExportAllocationRule for LegacyDataExportAllocationDurationMinutesRangeRule {
    fn check(&self, item: &LegacyDataExportAllocation) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy data export allocation.
#[derive(Debug, Clone)]
pub struct LegacyDataExportAllocationValidator {
    pub rules: Vec<Rc<dyn LegacyDataExportAllocationRule>>,
}

impl LegacyDataExportAllocationValidator {
    pub fn validate(&self, item: &LegacyDataExportAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyDataExportAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyDataExportAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_data_export_allocation_validator() -> LegacyDataExportAllocationValidator {
    LegacyDataExportAllocationValidator {
        rules: vec![
            Rc::new(LegacyDataExportAllocationSourceLengthRule { max_length: 32 })
                as Rc<dyn LegacyDataExportAllocationRule>,
            Rc::new(LegacyDataExportAllocationPhoneLengthRule { max_length: 1000 }),
            Rc::new(LegacyDataExportAllocationCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(LegacyDataExportAllocationBurstRangeRule { min: 32, max: 120 }),
            Rc::new(LegacyDataExportAllocationCodeLengthRule { max_length: 255 }),
            Rc::new(LegacyDataExportAllocationDurationMinutesRangeRule { min: 32, max: 86400 }),
        ],
    }
}
