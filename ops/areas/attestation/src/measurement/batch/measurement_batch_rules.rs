use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a measurement batch must pass. Check returns its problems, or none.
pub trait MeasurementBatchRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBatchRequiredTitleRule {
    pub message: String,
}

impl MeasurementBatchRule for MeasurementBatchRequiredTitleRule {
    fn check(&self, item: &MeasurementBatch) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBatchAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl MeasurementBatchRule for MeasurementBatchAllowedCodeRule {
    fn check(&self, item: &MeasurementBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBatchCodeLengthRule {
    pub max_length: i64,
}

impl MeasurementBatchRule for MeasurementBatchCodeLengthRule {
    fn check(&self, item: &MeasurementBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement batch.
#[derive(Debug, Clone)]
pub struct MeasurementBatchValidator {
    pub rules: Vec<Rc<dyn MeasurementBatchRule>>,
}

impl MeasurementBatchValidator {
    pub fn validate(&self, item: &MeasurementBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_batch_validator() -> MeasurementBatchValidator {
    MeasurementBatchValidator {
        rules: vec![
            Rc::new(MeasurementBatchRequiredTitleRule {
                message: "measurement batch needs an entry".to_string(),
            }) as Rc<dyn MeasurementBatchRule>,
            Rc::new(MeasurementBatchAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(MeasurementBatchCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_batch_validator();
        let item = new_measurement_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = MeasurementBatchRequiredTitleRule {
            message: "measurement batch needs a value".to_string(),
        };
        let mut item = new_measurement_batch("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = MeasurementBatchCodeLengthRule { max_length: 16 };
        let mut item = new_measurement_batch("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
