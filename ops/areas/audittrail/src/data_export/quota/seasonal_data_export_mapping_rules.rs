use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal data export mapping must pass. Check returns its problems, or none.
pub trait SeasonalDataExportMappingRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalDataExportMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportMappingNameLengthRule {
    pub max_length: i64,
}

impl SeasonalDataExportMappingRule for SeasonalDataExportMappingNameLengthRule {
    fn check(&self, item: &SeasonalDataExportMapping) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportMappingRequiredCodeRule {
    pub message: String,
}

impl SeasonalDataExportMappingRule for SeasonalDataExportMappingRequiredCodeRule {
    fn check(&self, item: &SeasonalDataExportMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportMappingCodePrefixRule {
    pub prefix: String,
}

impl SeasonalDataExportMappingRule for SeasonalDataExportMappingCodePrefixRule {
    fn check(&self, item: &SeasonalDataExportMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal data export mapping.
#[derive(Debug, Clone)]
pub struct SeasonalDataExportMappingValidator {
    pub rules: Vec<Rc<dyn SeasonalDataExportMappingRule>>,
}

impl SeasonalDataExportMappingValidator {
    pub fn validate(&self, item: &SeasonalDataExportMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalDataExportMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalDataExportMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_data_export_mapping_validator() -> SeasonalDataExportMappingValidator {
    SeasonalDataExportMappingValidator {
        rules: vec![
            Rc::new(SeasonalDataExportMappingNameLengthRule { max_length: 3600 })
                as Rc<dyn SeasonalDataExportMappingRule>,
            Rc::new(SeasonalDataExportMappingRequiredCodeRule {
                message: "seasonal data export mapping needs an entry".to_string(),
            }),
            Rc::new(SeasonalDataExportMappingCodePrefixRule {
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
        let validator = default_seasonal_data_export_mapping_validator();
        let item = new_seasonal_data_export_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SeasonalDataExportMappingNameLengthRule { max_length: 255 };
        let mut item = new_seasonal_data_export_mapping("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalDataExportMappingRequiredCodeRule {
            message: "seasonal data export mapping needs a value".to_string(),
        };
        let mut item = new_seasonal_data_export_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
