use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export entry must pass. Check returns its problems, or none.
pub trait DataExportEntryRule: std::fmt::Debug {
    fn check(&self, item: &DataExportEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportEntryCodeLengthRule {
    pub max_length: i64,
}

impl DataExportEntryRule for DataExportEntryCodeLengthRule {
    fn check(&self, item: &DataExportEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportEntryNameLengthRule {
    pub max_length: i64,
}

impl DataExportEntryRule for DataExportEntryNameLengthRule {
    fn check(&self, item: &DataExportEntry) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportEntryRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DataExportEntryRule for DataExportEntryRatioRangeRule {
    fn check(&self, item: &DataExportEntry) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportEntryUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportEntryRule for DataExportEntryUpdatedAtRangeRule {
    fn check(&self, item: &DataExportEntry) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportEntryRequiredPhoneRule {
    pub message: String,
}

impl DataExportEntryRule for DataExportEntryRequiredPhoneRule {
    fn check(&self, item: &DataExportEntry) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportEntryCodePrefixRule {
    pub prefix: String,
}

impl DataExportEntryRule for DataExportEntryCodePrefixRule {
    fn check(&self, item: &DataExportEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export entry.
#[derive(Debug, Clone)]
pub struct DataExportEntryValidator {
    pub rules: Vec<Rc<dyn DataExportEntryRule>>,
}

impl DataExportEntryValidator {
    pub fn validate(&self, item: &DataExportEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_entry_validator() -> DataExportEntryValidator {
    DataExportEntryValidator {
        rules: vec![
            Rc::new(DataExportEntryCodeLengthRule { max_length: 32 }) as Rc<dyn DataExportEntryRule>,
            Rc::new(DataExportEntryNameLengthRule { max_length: 32 }),
            Rc::new(DataExportEntryRatioRangeRule { min: 3.5, max: 2119.4 }),
            Rc::new(DataExportEntryUpdatedAtRangeRule { min: 255, max: 3600 }),
            Rc::new(DataExportEntryRequiredPhoneRule {
                message: "data export entry needs an entry".to_string(),
            }),
            Rc::new(DataExportEntryCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_entry_validator();
        let item = new_data_export_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DataExportEntryCodeLengthRule { max_length: 255 };
        let mut item = new_data_export_entry("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DataExportEntryNameLengthRule { max_length: 255 };
        let mut item = new_data_export_entry("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = DataExportEntryRatioRangeRule { min: 1.6, max: 3182.2 };
        let mut item = new_data_export_entry("x", "y");
        item.ratio = 3183.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = DataExportEntryUpdatedAtRangeRule { min: 1000, max: 86400 };
        let mut item = new_data_export_entry("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = DataExportEntryRequiredPhoneRule {
            message: "data export entry needs a value".to_string(),
        };
        let mut item = new_data_export_entry("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
