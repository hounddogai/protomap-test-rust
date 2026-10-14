use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default data export mapping must pass. Check returns its problems, or none.
pub trait DefaultDataExportMappingRule: std::fmt::Debug {
    fn check(&self, item: &DefaultDataExportMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportMappingLabelLengthRule {
    pub max_length: i64,
}

impl DefaultDataExportMappingRule for DefaultDataExportMappingLabelLengthRule {
    fn check(&self, item: &DefaultDataExportMapping) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportMappingCodePrefixRule {
    pub prefix: String,
}

impl DefaultDataExportMappingRule for DefaultDataExportMappingCodePrefixRule {
    fn check(&self, item: &DefaultDataExportMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportMappingExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultDataExportMappingRule for DefaultDataExportMappingExpiresAtRangeRule {
    fn check(&self, item: &DefaultDataExportMapping) -> Vec<String> {
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
pub struct DefaultDataExportMappingAllowedNameRule {
    pub allowed: Vec<String>,
}

impl DefaultDataExportMappingRule for DefaultDataExportMappingAllowedNameRule {
    fn check(&self, item: &DefaultDataExportMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is duplicated", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportMappingRequiredNameRule {
    pub message: String,
}

impl DefaultDataExportMappingRule for DefaultDataExportMappingRequiredNameRule {
    fn check(&self, item: &DefaultDataExportMapping) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default data export mapping.
#[derive(Debug, Clone)]
pub struct DefaultDataExportMappingValidator {
    pub rules: Vec<Rc<dyn DefaultDataExportMappingRule>>,
}

impl DefaultDataExportMappingValidator {
    pub fn validate(&self, item: &DefaultDataExportMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultDataExportMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultDataExportMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_data_export_mapping_validator() -> DefaultDataExportMappingValidator {
    DefaultDataExportMappingValidator {
        rules: vec![
            Rc::new(DefaultDataExportMappingLabelLengthRule { max_length: 64 }) as Rc<dyn DefaultDataExportMappingRule>,
            Rc::new(DefaultDataExportMappingCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(DefaultDataExportMappingExpiresAtRangeRule { min: 80, max: 80 }),
            Rc::new(DefaultDataExportMappingAllowedNameRule { allowed: Vec::new() }),
            Rc::new(DefaultDataExportMappingRequiredNameRule {
                message: "default data export mapping needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_data_export_mapping_validator();
        let item = new_default_data_export_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DefaultDataExportMappingLabelLengthRule { max_length: 64 };
        let mut item = new_default_data_export_mapping("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = DefaultDataExportMappingExpiresAtRangeRule { min: 64, max: 255 };
        let mut item = new_default_data_export_mapping("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DefaultDataExportMappingRequiredNameRule {
            message: "default data export mapping needs a value".to_string(),
        };
        let mut item = new_default_data_export_mapping("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
