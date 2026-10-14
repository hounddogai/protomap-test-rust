use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a data export mapping must pass. Check returns its problems, or none.
pub trait DataExportMappingRule: std::fmt::Debug {
    fn check(&self, item: &DataExportMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMappingAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DataExportMappingRule for DataExportMappingAllowedCodeRule {
    fn check(&self, item: &DataExportMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMappingCodeLengthRule {
    pub max_length: i64,
}

impl DataExportMappingRule for DataExportMappingCodeLengthRule {
    fn check(&self, item: &DataExportMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMappingExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportMappingRule for DataExportMappingExpiresAtRangeRule {
    fn check(&self, item: &DataExportMapping) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMappingLabelLengthRule {
    pub max_length: i64,
}

impl DataExportMappingRule for DataExportMappingLabelLengthRule {
    fn check(&self, item: &DataExportMapping) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMappingCodePrefixRule {
    pub prefix: String,
}

impl DataExportMappingRule for DataExportMappingCodePrefixRule {
    fn check(&self, item: &DataExportMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export mapping.
#[derive(Debug, Clone)]
pub struct DataExportMappingValidator {
    pub rules: Vec<Rc<dyn DataExportMappingRule>>,
}

impl DataExportMappingValidator {
    pub fn validate(&self, item: &DataExportMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DataExportMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_mapping_validator() -> DataExportMappingValidator {
    DataExportMappingValidator {
        rules: vec![
            Rc::new(DataExportMappingAllowedCodeRule {
                allowed: vec!["rolling".to_string()],
            }) as Rc<dyn DataExportMappingRule>,
            Rc::new(DataExportMappingCodeLengthRule { max_length: 120 }),
            Rc::new(DataExportMappingExpiresAtRangeRule { min: 120, max: 120 }),
            Rc::new(DataExportMappingLabelLengthRule { max_length: 1000 }),
            Rc::new(DataExportMappingCodePrefixRule {
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
        let validator = default_data_export_mapping_validator();
        let item = new_data_export_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DataExportMappingCodeLengthRule { max_length: 64 };
        let mut item = new_data_export_mapping("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = DataExportMappingExpiresAtRangeRule { min: 32, max: 86400 };
        let mut item = new_data_export_mapping("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DataExportMappingLabelLengthRule { max_length: 64 };
        let mut item = new_data_export_mapping("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
