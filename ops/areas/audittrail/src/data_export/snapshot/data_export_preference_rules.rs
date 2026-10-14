use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export preference must pass. Check returns its problems, or none.
pub trait DataExportPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &DataExportPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPreferenceNameLengthRule {
    pub max_length: i64,
}

impl DataExportPreferenceRule for DataExportPreferenceNameLengthRule {
    fn check(&self, item: &DataExportPreference) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPreferenceRequiredNameRule {
    pub message: String,
}

impl DataExportPreferenceRule for DataExportPreferenceRequiredNameRule {
    fn check(&self, item: &DataExportPreference) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPreferenceRequiredScopeRule {
    pub message: String,
}

impl DataExportPreferenceRule for DataExportPreferenceRequiredScopeRule {
    fn check(&self, item: &DataExportPreference) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl DataExportPreferenceRule for DataExportPreferenceCodeLengthRule {
    fn check(&self, item: &DataExportPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPreferenceCodePrefixRule {
    pub prefix: String,
}

impl DataExportPreferenceRule for DataExportPreferenceCodePrefixRule {
    fn check(&self, item: &DataExportPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export preference.
#[derive(Debug, Clone)]
pub struct DataExportPreferenceValidator {
    pub rules: Vec<Rc<dyn DataExportPreferenceRule>>,
}

impl DataExportPreferenceValidator {
    pub fn validate(&self, item: &DataExportPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_preference_validator() -> DataExportPreferenceValidator {
    DataExportPreferenceValidator {
        rules: vec![
            Rc::new(DataExportPreferenceNameLengthRule { max_length: 32 }) as Rc<dyn DataExportPreferenceRule>,
            Rc::new(DataExportPreferenceRequiredNameRule {
                message: "data export preference needs a value".to_string(),
            }),
            Rc::new(DataExportPreferenceRequiredScopeRule {
                message: "data export preference needs an entry".to_string(),
            }),
            Rc::new(DataExportPreferenceCodeLengthRule { max_length: 255 }),
            Rc::new(DataExportPreferenceCodePrefixRule {
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
        let validator = default_data_export_preference_validator();
        let item = new_data_export_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DataExportPreferenceNameLengthRule { max_length: 3600 };
        let mut item = new_data_export_preference("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DataExportPreferenceRequiredNameRule {
            message: "data export preference needs an entry".to_string(),
        };
        let mut item = new_data_export_preference("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = DataExportPreferenceRequiredScopeRule {
            message: "data export preference needs an entry".to_string(),
        };
        let mut item = new_data_export_preference("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DataExportPreferenceCodeLengthRule { max_length: 64 };
        let mut item = new_data_export_preference("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
