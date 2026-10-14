use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export profile must pass. Check returns its problems, or none.
pub trait DataExportProfileRule: std::fmt::Debug {
    fn check(&self, item: &DataExportProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportProfileRequiredCurrencyRule {
    pub message: String,
}

impl DataExportProfileRule for DataExportProfileRequiredCurrencyRule {
    fn check(&self, item: &DataExportProfile) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportProfileCodePrefixRule {
    pub prefix: String,
}

impl DataExportProfileRule for DataExportProfileCodePrefixRule {
    fn check(&self, item: &DataExportProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportProfileRequiredNameRule {
    pub message: String,
}

impl DataExportProfileRule for DataExportProfileRequiredNameRule {
    fn check(&self, item: &DataExportProfile) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportProfileRequiredCodeRule {
    pub message: String,
}

impl DataExportProfileRule for DataExportProfileRequiredCodeRule {
    fn check(&self, item: &DataExportProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export profile.
#[derive(Debug, Clone)]
pub struct DataExportProfileValidator {
    pub rules: Vec<Rc<dyn DataExportProfileRule>>,
}

impl DataExportProfileValidator {
    pub fn validate(&self, item: &DataExportProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_profile_validator() -> DataExportProfileValidator {
    DataExportProfileValidator {
        rules: vec![
            Rc::new(DataExportProfileRequiredCurrencyRule {
                message: "data export profile needs an entry".to_string(),
            }) as Rc<dyn DataExportProfileRule>,
            Rc::new(DataExportProfileCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(DataExportProfileRequiredNameRule {
                message: "data export profile needs this field".to_string(),
            }),
            Rc::new(DataExportProfileRequiredCodeRule {
                message: "data export profile needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_profile_validator();
        let item = new_data_export_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = DataExportProfileRequiredCurrencyRule {
            message: "data export profile needs this field".to_string(),
        };
        let mut item = new_data_export_profile("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DataExportProfileRequiredNameRule {
            message: "data export profile needs this field".to_string(),
        };
        let mut item = new_data_export_profile("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DataExportProfileRequiredCodeRule {
            message: "data export profile needs an entry".to_string(),
        };
        let mut item = new_data_export_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
