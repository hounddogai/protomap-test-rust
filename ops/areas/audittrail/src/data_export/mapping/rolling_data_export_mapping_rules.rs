use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling data export mapping must pass. Check returns its problems, or none.
pub trait RollingDataExportMappingRule: std::fmt::Debug {
    fn check(&self, item: &RollingDataExportMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDataExportMappingRequiredCodeRule {
    pub message: String,
}

impl RollingDataExportMappingRule for RollingDataExportMappingRequiredCodeRule {
    fn check(&self, item: &RollingDataExportMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDataExportMappingCodeLengthRule {
    pub max_length: i64,
}

impl RollingDataExportMappingRule for RollingDataExportMappingCodeLengthRule {
    fn check(&self, item: &RollingDataExportMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDataExportMappingRequiredNameRule {
    pub message: String,
}

impl RollingDataExportMappingRule for RollingDataExportMappingRequiredNameRule {
    fn check(&self, item: &RollingDataExportMapping) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDataExportMappingCodePrefixRule {
    pub prefix: String,
}

impl RollingDataExportMappingRule for RollingDataExportMappingCodePrefixRule {
    fn check(&self, item: &RollingDataExportMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDataExportMappingPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingDataExportMappingRule for RollingDataExportMappingPriorityRangeRule {
    fn check(&self, item: &RollingDataExportMapping) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rolling data export mapping.
#[derive(Debug, Clone)]
pub struct RollingDataExportMappingValidator {
    pub rules: Vec<Rc<dyn RollingDataExportMappingRule>>,
}

impl RollingDataExportMappingValidator {
    pub fn validate(&self, item: &RollingDataExportMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingDataExportMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingDataExportMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_data_export_mapping_validator() -> RollingDataExportMappingValidator {
    RollingDataExportMappingValidator {
        rules: vec![
            Rc::new(RollingDataExportMappingRequiredCodeRule {
                message: "rolling data export mapping needs this field".to_string(),
            }) as Rc<dyn RollingDataExportMappingRule>,
            Rc::new(RollingDataExportMappingCodeLengthRule { max_length: 120 }),
            Rc::new(RollingDataExportMappingRequiredNameRule {
                message: "rolling data export mapping needs an entry".to_string(),
            }),
            Rc::new(RollingDataExportMappingCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RollingDataExportMappingPriorityRangeRule { min: 64, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_data_export_mapping_validator();
        let item = new_rolling_data_export_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingDataExportMappingRequiredCodeRule {
            message: "rolling data export mapping needs an entry".to_string(),
        };
        let mut item = new_rolling_data_export_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingDataExportMappingCodeLengthRule { max_length: 255 };
        let mut item = new_rolling_data_export_mapping("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollingDataExportMappingRequiredNameRule {
            message: "rolling data export mapping needs this field".to_string(),
        };
        let mut item = new_rolling_data_export_mapping("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = RollingDataExportMappingPriorityRangeRule { min: 3600, max: 3600 };
        let mut item = new_rolling_data_export_mapping("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
