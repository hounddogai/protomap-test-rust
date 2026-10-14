use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export quota must pass. Check returns its problems, or none.
pub trait DataExportQuotaRule: std::fmt::Debug {
    fn check(&self, item: &DataExportQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportQuotaRequiredClusterRule {
    pub message: String,
}

impl DataExportQuotaRule for DataExportQuotaRequiredClusterRule {
    fn check(&self, item: &DataExportQuota) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportQuotaNameLengthRule {
    pub max_length: i64,
}

impl DataExportQuotaRule for DataExportQuotaNameLengthRule {
    fn check(&self, item: &DataExportQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportQuotaRequiredKeyIdRule {
    pub message: String,
}

impl DataExportQuotaRule for DataExportQuotaRequiredKeyIdRule {
    fn check(&self, item: &DataExportQuota) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportQuotaSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportQuotaRule for DataExportQuotaSequenceRangeRule {
    fn check(&self, item: &DataExportQuota) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportQuotaCodePrefixRule {
    pub prefix: String,
}

impl DataExportQuotaRule for DataExportQuotaCodePrefixRule {
    fn check(&self, item: &DataExportQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export quota.
#[derive(Debug, Clone)]
pub struct DataExportQuotaValidator {
    pub rules: Vec<Rc<dyn DataExportQuotaRule>>,
}

impl DataExportQuotaValidator {
    pub fn validate(&self, item: &DataExportQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DataExportQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_quota_validator() -> DataExportQuotaValidator {
    DataExportQuotaValidator {
        rules: vec![
            Rc::new(DataExportQuotaRequiredClusterRule {
                message: "data export quota needs this field".to_string(),
            }) as Rc<dyn DataExportQuotaRule>,
            Rc::new(DataExportQuotaNameLengthRule { max_length: 255 }),
            Rc::new(DataExportQuotaRequiredKeyIdRule {
                message: "data export quota needs an entry".to_string(),
            }),
            Rc::new(DataExportQuotaSequenceRangeRule { min: 64, max: 3600 }),
            Rc::new(DataExportQuotaCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
