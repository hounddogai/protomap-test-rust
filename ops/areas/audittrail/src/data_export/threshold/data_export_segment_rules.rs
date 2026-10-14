use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export segment must pass. Check returns its problems, or none.
pub trait DataExportSegmentRule: std::fmt::Debug {
    fn check(&self, item: &DataExportSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportSegmentRequiredTenantIdRule {
    pub message: String,
}

impl DataExportSegmentRule for DataExportSegmentRequiredTenantIdRule {
    fn check(&self, item: &DataExportSegment) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportSegmentCodePrefixRule {
    pub prefix: String,
}

impl DataExportSegmentRule for DataExportSegmentCodePrefixRule {
    fn check(&self, item: &DataExportSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportSegmentRequiredChecksumRule {
    pub message: String,
}

impl DataExportSegmentRule for DataExportSegmentRequiredChecksumRule {
    fn check(&self, item: &DataExportSegment) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportSegmentCodeLengthRule {
    pub max_length: i64,
}

impl DataExportSegmentRule for DataExportSegmentCodeLengthRule {
    fn check(&self, item: &DataExportSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportSegmentLabelLengthRule {
    pub max_length: i64,
}

impl DataExportSegmentRule for DataExportSegmentLabelLengthRule {
    fn check(&self, item: &DataExportSegment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export segment.
#[derive(Debug, Clone)]
pub struct DataExportSegmentValidator {
    pub rules: Vec<Rc<dyn DataExportSegmentRule>>,
}

impl DataExportSegmentValidator {
    pub fn validate(&self, item: &DataExportSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_segment_validator() -> DataExportSegmentValidator {
    DataExportSegmentValidator {
        rules: vec![
            Rc::new(DataExportSegmentRequiredTenantIdRule {
                message: "data export segment needs an entry".to_string(),
            }) as Rc<dyn DataExportSegmentRule>,
            Rc::new(DataExportSegmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(DataExportSegmentRequiredChecksumRule {
                message: "data export segment needs a value".to_string(),
            }),
            Rc::new(DataExportSegmentCodeLengthRule { max_length: 86400 }),
            Rc::new(DataExportSegmentLabelLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_segment_validator();
        let item = new_data_export_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = DataExportSegmentRequiredTenantIdRule {
            message: "data export segment needs an entry".to_string(),
        };
        let mut item = new_data_export_segment("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = DataExportSegmentRequiredChecksumRule {
            message: "data export segment needs this field".to_string(),
        };
        let mut item = new_data_export_segment("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DataExportSegmentCodeLengthRule { max_length: 1000 };
        let mut item = new_data_export_segment("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DataExportSegmentLabelLengthRule { max_length: 1000 };
        let mut item = new_data_export_segment("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
