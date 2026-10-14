use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export draft must pass. Check returns its problems, or none.
pub trait DataExportDraftRule: std::fmt::Debug {
    fn check(&self, item: &DataExportDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportDraftVisibleGuardRule {
    pub limit: f64,
}

impl DataExportDraftRule for DataExportDraftVisibleGuardRule {
    fn check(&self, item: &DataExportDraft) -> Vec<String> {
        if item.visible && (item.threshold > self.limit) {
            return vec![format!(
                "threshold {} exceeds {} while visible",
                item.threshold, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportDraftRegionLengthRule {
    pub max_length: i64,
}

impl DataExportDraftRule for DataExportDraftRegionLengthRule {
    fn check(&self, item: &DataExportDraft) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportDraftCodeLengthRule {
    pub max_length: i64,
}

impl DataExportDraftRule for DataExportDraftCodeLengthRule {
    fn check(&self, item: &DataExportDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportDraftCodePrefixRule {
    pub prefix: String,
}

impl DataExportDraftRule for DataExportDraftCodePrefixRule {
    fn check(&self, item: &DataExportDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportDraftRequiredNameRule {
    pub message: String,
}

impl DataExportDraftRule for DataExportDraftRequiredNameRule {
    fn check(&self, item: &DataExportDraft) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportDraftThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DataExportDraftRule for DataExportDraftThresholdRangeRule {
    fn check(&self, item: &DataExportDraft) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export draft.
#[derive(Debug, Clone)]
pub struct DataExportDraftValidator {
    pub rules: Vec<Rc<dyn DataExportDraftRule>>,
}

impl DataExportDraftValidator {
    pub fn validate(&self, item: &DataExportDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DataExportDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_draft_validator() -> DataExportDraftValidator {
    DataExportDraftValidator {
        rules: vec![
            Rc::new(DataExportDraftVisibleGuardRule { limit: 1.7 }) as Rc<dyn DataExportDraftRule>,
            Rc::new(DataExportDraftRegionLengthRule { max_length: 120 }),
            Rc::new(DataExportDraftCodeLengthRule { max_length: 64 }),
            Rc::new(DataExportDraftCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(DataExportDraftRequiredNameRule {
                message: "data export draft needs an entry".to_string(),
            }),
            Rc::new(DataExportDraftThresholdRangeRule { min: 7.9, max: 2308.0 }),
        ],
    }
}
