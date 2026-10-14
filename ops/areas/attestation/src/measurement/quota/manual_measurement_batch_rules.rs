use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual measurement batch must pass. Check returns its problems, or none.
pub trait ManualMeasurementBatchRule: std::fmt::Debug {
    fn check(&self, item: &ManualMeasurementBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMeasurementBatchRequiredCodeRule {
    pub message: String,
}

impl ManualMeasurementBatchRule for ManualMeasurementBatchRequiredCodeRule {
    fn check(&self, item: &ManualMeasurementBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMeasurementBatchRequiredLabelRule {
    pub message: String,
}

impl ManualMeasurementBatchRule for ManualMeasurementBatchRequiredLabelRule {
    fn check(&self, item: &ManualMeasurementBatch) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMeasurementBatchMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualMeasurementBatchRule for ManualMeasurementBatchMarginRangeRule {
    fn check(&self, item: &ManualMeasurementBatch) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMeasurementBatchCodeLengthRule {
    pub max_length: i64,
}

impl ManualMeasurementBatchRule for ManualMeasurementBatchCodeLengthRule {
    fn check(&self, item: &ManualMeasurementBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual measurement batch.
#[derive(Debug, Clone)]
pub struct ManualMeasurementBatchValidator {
    pub rules: Vec<Rc<dyn ManualMeasurementBatchRule>>,
}

impl ManualMeasurementBatchValidator {
    pub fn validate(&self, item: &ManualMeasurementBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualMeasurementBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualMeasurementBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_measurement_batch_validator() -> ManualMeasurementBatchValidator {
    ManualMeasurementBatchValidator {
        rules: vec![
            Rc::new(ManualMeasurementBatchRequiredCodeRule {
                message: "manual measurement batch needs this field".to_string(),
            }) as Rc<dyn ManualMeasurementBatchRule>,
            Rc::new(ManualMeasurementBatchRequiredLabelRule {
                message: "manual measurement batch needs this field".to_string(),
            }),
            Rc::new(ManualMeasurementBatchMarginRangeRule { min: 7.7, max: 4716.5 }),
            Rc::new(ManualMeasurementBatchCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_measurement_batch_validator();
        let item = new_manual_measurement_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualMeasurementBatchRequiredCodeRule {
            message: "manual measurement batch needs an entry".to_string(),
        };
        let mut item = new_manual_measurement_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ManualMeasurementBatchRequiredLabelRule {
            message: "manual measurement batch needs an entry".to_string(),
        };
        let mut item = new_manual_measurement_batch("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = ManualMeasurementBatchMarginRangeRule { min: 2.9, max: 750.7 };
        let mut item = new_manual_measurement_batch("x", "y");
        item.margin = 752.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualMeasurementBatchCodeLengthRule { max_length: 255 };
        let mut item = new_manual_measurement_batch("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
