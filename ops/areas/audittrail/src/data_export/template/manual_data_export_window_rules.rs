use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual data export window must pass. Check returns its problems, or none.
pub trait ManualDataExportWindowRule: std::fmt::Debug {
    fn check(&self, item: &ManualDataExportWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportWindowRequiredTitleRule {
    pub message: String,
}

impl ManualDataExportWindowRule for ManualDataExportWindowRequiredTitleRule {
    fn check(&self, item: &ManualDataExportWindow) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportWindowArchivedGuardRule {
    pub limit: i64,
}

impl ManualDataExportWindowRule for ManualDataExportWindowArchivedGuardRule {
    fn check(&self, item: &ManualDataExportWindow) -> Vec<String> {
        if item.archived && (item.ttl_seconds > self.limit) {
            return vec![format!(
                "ttl seconds {} exceeds {} while archived",
                item.ttl_seconds, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportWindowRequiredCodeRule {
    pub message: String,
}

impl ManualDataExportWindowRule for ManualDataExportWindowRequiredCodeRule {
    fn check(&self, item: &ManualDataExportWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportWindowQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualDataExportWindowRule for ManualDataExportWindowQuotaUnitsRangeRule {
    fn check(&self, item: &ManualDataExportWindow) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a manual data export window.
#[derive(Debug, Clone)]
pub struct ManualDataExportWindowValidator {
    pub rules: Vec<Rc<dyn ManualDataExportWindowRule>>,
}

impl ManualDataExportWindowValidator {
    pub fn validate(&self, item: &ManualDataExportWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualDataExportWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualDataExportWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_data_export_window_validator() -> ManualDataExportWindowValidator {
    ManualDataExportWindowValidator {
        rules: vec![
            Rc::new(ManualDataExportWindowRequiredTitleRule {
                message: "manual data export window needs an entry".to_string(),
            }) as Rc<dyn ManualDataExportWindowRule>,
            Rc::new(ManualDataExportWindowArchivedGuardRule { limit: 255 }),
            Rc::new(ManualDataExportWindowRequiredCodeRule {
                message: "manual data export window needs a value".to_string(),
            }),
            Rc::new(ManualDataExportWindowQuotaUnitsRangeRule { min: 32, max: 80 }),
        ],
    }
}
