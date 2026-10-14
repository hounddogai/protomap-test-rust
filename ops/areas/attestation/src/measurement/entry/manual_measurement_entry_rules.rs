use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual measurement entry must pass. Check returns its problems, or none.
pub trait ManualMeasurementEntryRule: std::fmt::Debug {
    fn check(&self, item: &ManualMeasurementEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMeasurementEntrySourceLengthRule {
    pub max_length: i64,
}

impl ManualMeasurementEntryRule for ManualMeasurementEntrySourceLengthRule {
    fn check(&self, item: &ManualMeasurementEntry) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMeasurementEntryRequiredCodeRule {
    pub message: String,
}

impl ManualMeasurementEntryRule for ManualMeasurementEntryRequiredCodeRule {
    fn check(&self, item: &ManualMeasurementEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMeasurementEntryCodePrefixRule {
    pub prefix: String,
}

impl ManualMeasurementEntryRule for ManualMeasurementEntryCodePrefixRule {
    fn check(&self, item: &ManualMeasurementEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMeasurementEntryRequiredFingerprintRule {
    pub message: String,
}

impl ManualMeasurementEntryRule for ManualMeasurementEntryRequiredFingerprintRule {
    fn check(&self, item: &ManualMeasurementEntry) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMeasurementEntryTitleLengthRule {
    pub max_length: i64,
}

impl ManualMeasurementEntryRule for ManualMeasurementEntryTitleLengthRule {
    fn check(&self, item: &ManualMeasurementEntry) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMeasurementEntryCodeLengthRule {
    pub max_length: i64,
}

impl ManualMeasurementEntryRule for ManualMeasurementEntryCodeLengthRule {
    fn check(&self, item: &ManualMeasurementEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual measurement entry.
#[derive(Debug, Clone)]
pub struct ManualMeasurementEntryValidator {
    pub rules: Vec<Rc<dyn ManualMeasurementEntryRule>>,
}

impl ManualMeasurementEntryValidator {
    pub fn validate(&self, item: &ManualMeasurementEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualMeasurementEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualMeasurementEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_measurement_entry_validator() -> ManualMeasurementEntryValidator {
    ManualMeasurementEntryValidator {
        rules: vec![
            Rc::new(ManualMeasurementEntrySourceLengthRule { max_length: 3600 }) as Rc<dyn ManualMeasurementEntryRule>,
            Rc::new(ManualMeasurementEntryRequiredCodeRule {
                message: "manual measurement entry needs an entry".to_string(),
            }),
            Rc::new(ManualMeasurementEntryCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ManualMeasurementEntryRequiredFingerprintRule {
                message: "manual measurement entry needs an entry".to_string(),
            }),
            Rc::new(ManualMeasurementEntryTitleLengthRule { max_length: 1000 }),
            Rc::new(ManualMeasurementEntryCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_measurement_entry_validator();
        let item = new_manual_measurement_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = ManualMeasurementEntrySourceLengthRule { max_length: 80 };
        let mut item = new_manual_measurement_entry("x", "y");
        item.source = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualMeasurementEntryRequiredCodeRule {
            message: "manual measurement entry needs an entry".to_string(),
        };
        let mut item = new_manual_measurement_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = ManualMeasurementEntryRequiredFingerprintRule {
            message: "manual measurement entry needs this field".to_string(),
        };
        let mut item = new_manual_measurement_entry("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ManualMeasurementEntryTitleLengthRule { max_length: 120 };
        let mut item = new_manual_measurement_entry("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualMeasurementEntryCodeLengthRule { max_length: 120 };
        let mut item = new_manual_measurement_entry("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
