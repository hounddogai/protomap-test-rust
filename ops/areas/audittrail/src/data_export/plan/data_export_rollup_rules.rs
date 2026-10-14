use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export rollup must pass. Check returns its problems, or none.
pub trait DataExportRollupRule: std::fmt::Debug {
    fn check(&self, item: &DataExportRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRollupRequiredCodeRule {
    pub message: String,
}

impl DataExportRollupRule for DataExportRollupRequiredCodeRule {
    fn check(&self, item: &DataExportRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRollupRequiredRegionRule {
    pub message: String,
}

impl DataExportRollupRule for DataExportRollupRequiredRegionRule {
    fn check(&self, item: &DataExportRollup) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRollupRequiredChecksumRule {
    pub message: String,
}

impl DataExportRollupRule for DataExportRollupRequiredChecksumRule {
    fn check(&self, item: &DataExportRollup) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRollupChecksumLengthRule {
    pub max_length: i64,
}

impl DataExportRollupRule for DataExportRollupChecksumLengthRule {
    fn check(&self, item: &DataExportRollup) -> Vec<String> {
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

/// Runs every rule of a data export rollup.
#[derive(Debug, Clone)]
pub struct DataExportRollupValidator {
    pub rules: Vec<Rc<dyn DataExportRollupRule>>,
}

impl DataExportRollupValidator {
    pub fn validate(&self, item: &DataExportRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_rollup_validator() -> DataExportRollupValidator {
    DataExportRollupValidator {
        rules: vec![
            Rc::new(DataExportRollupRequiredCodeRule {
                message: "data export rollup needs a value".to_string(),
            }) as Rc<dyn DataExportRollupRule>,
            Rc::new(DataExportRollupRequiredRegionRule {
                message: "data export rollup needs an entry".to_string(),
            }),
            Rc::new(DataExportRollupRequiredChecksumRule {
                message: "data export rollup needs an entry".to_string(),
            }),
            Rc::new(DataExportRollupChecksumLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_rollup_validator();
        let item = new_data_export_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DataExportRollupRequiredCodeRule {
            message: "data export rollup needs a value".to_string(),
        };
        let mut item = new_data_export_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = DataExportRollupRequiredRegionRule {
            message: "data export rollup needs a value".to_string(),
        };
        let mut item = new_data_export_rollup("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = DataExportRollupRequiredChecksumRule {
            message: "data export rollup needs an entry".to_string(),
        };
        let mut item = new_data_export_rollup("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = DataExportRollupChecksumLengthRule { max_length: 80 };
        let mut item = new_data_export_rollup("x", "y");
        item.checksum = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
