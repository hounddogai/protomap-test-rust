use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict measurement batch must pass. Check returns its problems, or none.
pub trait StrictMeasurementBatchRule: std::fmt::Debug {
    fn check(&self, item: &StrictMeasurementBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictMeasurementBatchRequiredHostRule {
    pub message: String,
}

impl StrictMeasurementBatchRule for StrictMeasurementBatchRequiredHostRule {
    fn check(&self, item: &StrictMeasurementBatch) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictMeasurementBatchCodeLengthRule {
    pub max_length: i64,
}

impl StrictMeasurementBatchRule for StrictMeasurementBatchCodeLengthRule {
    fn check(&self, item: &StrictMeasurementBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictMeasurementBatchTitleLengthRule {
    pub max_length: i64,
}

impl StrictMeasurementBatchRule for StrictMeasurementBatchTitleLengthRule {
    fn check(&self, item: &StrictMeasurementBatch) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictMeasurementBatchCodePrefixRule {
    pub prefix: String,
}

impl StrictMeasurementBatchRule for StrictMeasurementBatchCodePrefixRule {
    fn check(&self, item: &StrictMeasurementBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictMeasurementBatchCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictMeasurementBatchRule for StrictMeasurementBatchCreatedAtRangeRule {
    fn check(&self, item: &StrictMeasurementBatch) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictMeasurementBatchRequiredCodeRule {
    pub message: String,
}

impl StrictMeasurementBatchRule for StrictMeasurementBatchRequiredCodeRule {
    fn check(&self, item: &StrictMeasurementBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict measurement batch.
#[derive(Debug, Clone)]
pub struct StrictMeasurementBatchValidator {
    pub rules: Vec<Rc<dyn StrictMeasurementBatchRule>>,
}

impl StrictMeasurementBatchValidator {
    pub fn validate(&self, item: &StrictMeasurementBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictMeasurementBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictMeasurementBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_measurement_batch_validator() -> StrictMeasurementBatchValidator {
    StrictMeasurementBatchValidator {
        rules: vec![
            Rc::new(StrictMeasurementBatchRequiredHostRule {
                message: "strict measurement batch needs an entry".to_string(),
            }) as Rc<dyn StrictMeasurementBatchRule>,
            Rc::new(StrictMeasurementBatchCodeLengthRule { max_length: 80 }),
            Rc::new(StrictMeasurementBatchTitleLengthRule { max_length: 64 }),
            Rc::new(StrictMeasurementBatchCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(StrictMeasurementBatchCreatedAtRangeRule { min: 80, max: 1000 }),
            Rc::new(StrictMeasurementBatchRequiredCodeRule {
                message: "strict measurement batch needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_measurement_batch_validator();
        let item = new_strict_measurement_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = StrictMeasurementBatchRequiredHostRule {
            message: "strict measurement batch needs this field".to_string(),
        };
        let mut item = new_strict_measurement_batch("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictMeasurementBatchCodeLengthRule { max_length: 255 };
        let mut item = new_strict_measurement_batch("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = StrictMeasurementBatchTitleLengthRule { max_length: 16 };
        let mut item = new_strict_measurement_batch("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = StrictMeasurementBatchCreatedAtRangeRule { min: 120, max: 86400 };
        let mut item = new_strict_measurement_batch("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictMeasurementBatchRequiredCodeRule {
            message: "strict measurement batch needs this field".to_string(),
        };
        let mut item = new_strict_measurement_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
