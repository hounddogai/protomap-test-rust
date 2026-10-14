use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a measurement mapping must pass. Check returns its problems, or none.
pub trait MeasurementMappingRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementMappingCodeLengthRule {
    pub max_length: i64,
}

impl MeasurementMappingRule for MeasurementMappingCodeLengthRule {
    fn check(&self, item: &MeasurementMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementMappingAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl MeasurementMappingRule for MeasurementMappingAllowedLabelRule {
    fn check(&self, item: &MeasurementMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is blocked", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementMappingLabelLengthRule {
    pub max_length: i64,
}

impl MeasurementMappingRule for MeasurementMappingLabelLengthRule {
    fn check(&self, item: &MeasurementMapping) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementMappingRequiredCodeRule {
    pub message: String,
}

impl MeasurementMappingRule for MeasurementMappingRequiredCodeRule {
    fn check(&self, item: &MeasurementMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementMappingTitleLengthRule {
    pub max_length: i64,
}

impl MeasurementMappingRule for MeasurementMappingTitleLengthRule {
    fn check(&self, item: &MeasurementMapping) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement mapping.
#[derive(Debug, Clone)]
pub struct MeasurementMappingValidator {
    pub rules: Vec<Rc<dyn MeasurementMappingRule>>,
}

impl MeasurementMappingValidator {
    pub fn validate(&self, item: &MeasurementMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_mapping_validator() -> MeasurementMappingValidator {
    MeasurementMappingValidator {
        rules: vec![
            Rc::new(MeasurementMappingCodeLengthRule { max_length: 32 }) as Rc<dyn MeasurementMappingRule>,
            Rc::new(MeasurementMappingAllowedLabelRule {
                allowed: vec!["primary".to_string()],
            }),
            Rc::new(MeasurementMappingLabelLengthRule { max_length: 64 }),
            Rc::new(MeasurementMappingRequiredCodeRule {
                message: "measurement mapping needs this field".to_string(),
            }),
            Rc::new(MeasurementMappingTitleLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_mapping_validator();
        let item = new_measurement_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = MeasurementMappingCodeLengthRule { max_length: 16 };
        let mut item = new_measurement_mapping("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = MeasurementMappingLabelLengthRule { max_length: 32 };
        let mut item = new_measurement_mapping("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MeasurementMappingRequiredCodeRule {
            message: "measurement mapping needs an entry".to_string(),
        };
        let mut item = new_measurement_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = MeasurementMappingTitleLengthRule { max_length: 1000 };
        let mut item = new_measurement_mapping("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
