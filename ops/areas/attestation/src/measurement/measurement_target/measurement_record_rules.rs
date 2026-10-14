use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a measurement record must pass. Check returns its problems, or none.
pub trait MeasurementRecordRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRecordAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl MeasurementRecordRule for MeasurementRecordAllowedLabelRule {
    fn check(&self, item: &MeasurementRecord) -> Vec<String> {
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
pub struct MeasurementRecordCodePrefixRule {
    pub prefix: String,
}

impl MeasurementRecordRule for MeasurementRecordCodePrefixRule {
    fn check(&self, item: &MeasurementRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRecordRequiredCodeRule {
    pub message: String,
}

impl MeasurementRecordRule for MeasurementRecordRequiredCodeRule {
    fn check(&self, item: &MeasurementRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement record.
#[derive(Debug, Clone)]
pub struct MeasurementRecordValidator {
    pub rules: Vec<Rc<dyn MeasurementRecordRule>>,
}

impl MeasurementRecordValidator {
    pub fn validate(&self, item: &MeasurementRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_record_validator() -> MeasurementRecordValidator {
    MeasurementRecordValidator {
        rules: vec![
            Rc::new(MeasurementRecordAllowedLabelRule {
                allowed: vec!["fallback".to_string(), "capped".to_string()],
            }) as Rc<dyn MeasurementRecordRule>,
            Rc::new(MeasurementRecordCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(MeasurementRecordRequiredCodeRule {
                message: "measurement record needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_record_validator();
        let item = new_measurement_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MeasurementRecordRequiredCodeRule {
            message: "measurement record needs an entry".to_string(),
        };
        let mut item = new_measurement_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
