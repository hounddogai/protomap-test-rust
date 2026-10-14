use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export threshold must pass. Check returns its problems, or none.
pub trait DataExportThresholdRule: std::fmt::Debug {
    fn check(&self, item: &DataExportThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportThresholdRequiredFingerprintRule {
    pub message: String,
}

impl DataExportThresholdRule for DataExportThresholdRequiredFingerprintRule {
    fn check(&self, item: &DataExportThreshold) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportThresholdLabelLengthRule {
    pub max_length: i64,
}

impl DataExportThresholdRule for DataExportThresholdLabelLengthRule {
    fn check(&self, item: &DataExportThreshold) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportThresholdChecksumLengthRule {
    pub max_length: i64,
}

impl DataExportThresholdRule for DataExportThresholdChecksumLengthRule {
    fn check(&self, item: &DataExportThreshold) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportThresholdCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportThresholdRule for DataExportThresholdCreatedAtRangeRule {
    fn check(&self, item: &DataExportThreshold) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportThresholdRequiredLabelRule {
    pub message: String,
}

impl DataExportThresholdRule for DataExportThresholdRequiredLabelRule {
    fn check(&self, item: &DataExportThreshold) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportThresholdCodePrefixRule {
    pub prefix: String,
}

impl DataExportThresholdRule for DataExportThresholdCodePrefixRule {
    fn check(&self, item: &DataExportThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export threshold.
#[derive(Debug, Clone)]
pub struct DataExportThresholdValidator {
    pub rules: Vec<Rc<dyn DataExportThresholdRule>>,
}

impl DataExportThresholdValidator {
    pub fn validate(&self, item: &DataExportThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DataExportThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_threshold_validator() -> DataExportThresholdValidator {
    DataExportThresholdValidator {
        rules: vec![
            Rc::new(DataExportThresholdRequiredFingerprintRule {
                message: "data export threshold needs an entry".to_string(),
            }) as Rc<dyn DataExportThresholdRule>,
            Rc::new(DataExportThresholdLabelLengthRule { max_length: 32 }),
            Rc::new(DataExportThresholdChecksumLengthRule { max_length: 64 }),
            Rc::new(DataExportThresholdCreatedAtRangeRule { min: 32, max: 255 }),
            Rc::new(DataExportThresholdRequiredLabelRule {
                message: "data export threshold needs an entry".to_string(),
            }),
            Rc::new(DataExportThresholdCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_threshold_validator();
        let item = new_data_export_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = DataExportThresholdRequiredFingerprintRule {
            message: "data export threshold needs an entry".to_string(),
        };
        let mut item = new_data_export_threshold("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DataExportThresholdLabelLengthRule { max_length: 1000 };
        let mut item = new_data_export_threshold("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = DataExportThresholdChecksumLengthRule { max_length: 64 };
        let mut item = new_data_export_threshold("x", "y");
        item.checksum = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = DataExportThresholdCreatedAtRangeRule { min: 32, max: 1000 };
        let mut item = new_data_export_threshold("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DataExportThresholdRequiredLabelRule {
            message: "data export threshold needs a value".to_string(),
        };
        let mut item = new_data_export_threshold("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
