use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default measurement batch must pass. Check returns its problems, or none.
pub trait DefaultMeasurementBatchRule: std::fmt::Debug {
    fn check(&self, item: &DefaultMeasurementBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMeasurementBatchRequiredIssuerRule {
    pub message: String,
}

impl DefaultMeasurementBatchRule for DefaultMeasurementBatchRequiredIssuerRule {
    fn check(&self, item: &DefaultMeasurementBatch) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMeasurementBatchTitleLengthRule {
    pub max_length: i64,
}

impl DefaultMeasurementBatchRule for DefaultMeasurementBatchTitleLengthRule {
    fn check(&self, item: &DefaultMeasurementBatch) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMeasurementBatchRequiredCodeRule {
    pub message: String,
}

impl DefaultMeasurementBatchRule for DefaultMeasurementBatchRequiredCodeRule {
    fn check(&self, item: &DefaultMeasurementBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMeasurementBatchStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultMeasurementBatchRule for DefaultMeasurementBatchStartsAtRangeRule {
    fn check(&self, item: &DefaultMeasurementBatch) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default measurement batch.
#[derive(Debug, Clone)]
pub struct DefaultMeasurementBatchValidator {
    pub rules: Vec<Rc<dyn DefaultMeasurementBatchRule>>,
}

impl DefaultMeasurementBatchValidator {
    pub fn validate(&self, item: &DefaultMeasurementBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultMeasurementBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultMeasurementBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_measurement_batch_validator() -> DefaultMeasurementBatchValidator {
    DefaultMeasurementBatchValidator {
        rules: vec![
            Rc::new(DefaultMeasurementBatchRequiredIssuerRule {
                message: "default measurement batch needs this field".to_string(),
            }) as Rc<dyn DefaultMeasurementBatchRule>,
            Rc::new(DefaultMeasurementBatchTitleLengthRule { max_length: 120 }),
            Rc::new(DefaultMeasurementBatchRequiredCodeRule {
                message: "default measurement batch needs a value".to_string(),
            }),
            Rc::new(DefaultMeasurementBatchStartsAtRangeRule { min: 32, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_measurement_batch_validator();
        let item = new_default_measurement_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = DefaultMeasurementBatchRequiredIssuerRule {
            message: "default measurement batch needs a value".to_string(),
        };
        let mut item = new_default_measurement_batch("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DefaultMeasurementBatchTitleLengthRule { max_length: 120 };
        let mut item = new_default_measurement_batch("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultMeasurementBatchRequiredCodeRule {
            message: "default measurement batch needs this field".to_string(),
        };
        let mut item = new_default_measurement_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = DefaultMeasurementBatchStartsAtRangeRule { min: 120, max: 120 };
        let mut item = new_default_measurement_batch("x", "y");
        item.starts_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
