use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal data export snapshot must pass. Check returns its problems, or none.
pub trait SeasonalDataExportSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalDataExportSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportSnapshotRequiredTitleRule {
    pub message: String,
}

impl SeasonalDataExportSnapshotRule for SeasonalDataExportSnapshotRequiredTitleRule {
    fn check(&self, item: &SeasonalDataExportSnapshot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportSnapshotAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl SeasonalDataExportSnapshotRule for SeasonalDataExportSnapshotAllowedScopeRule {
    fn check(&self, item: &SeasonalDataExportSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is above maximum", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportSnapshotTitleLengthRule {
    pub max_length: i64,
}

impl SeasonalDataExportSnapshotRule for SeasonalDataExportSnapshotTitleLengthRule {
    fn check(&self, item: &SeasonalDataExportSnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalDataExportSnapshotRule for SeasonalDataExportSnapshotCodeLengthRule {
    fn check(&self, item: &SeasonalDataExportSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal data export snapshot.
#[derive(Debug, Clone)]
pub struct SeasonalDataExportSnapshotValidator {
    pub rules: Vec<Rc<dyn SeasonalDataExportSnapshotRule>>,
}

impl SeasonalDataExportSnapshotValidator {
    pub fn validate(&self, item: &SeasonalDataExportSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalDataExportSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalDataExportSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_data_export_snapshot_validator() -> SeasonalDataExportSnapshotValidator {
    SeasonalDataExportSnapshotValidator {
        rules: vec![
            Rc::new(SeasonalDataExportSnapshotRequiredTitleRule {
                message: "seasonal data export snapshot needs an entry".to_string(),
            }) as Rc<dyn SeasonalDataExportSnapshotRule>,
            Rc::new(SeasonalDataExportSnapshotAllowedScopeRule {
                allowed: vec!["legacy".to_string(), "rolling".to_string(), "adaptive".to_string()],
            }),
            Rc::new(SeasonalDataExportSnapshotTitleLengthRule { max_length: 86400 }),
            Rc::new(SeasonalDataExportSnapshotCodeLengthRule { max_length: 3600 }),
        ],
    }
}
