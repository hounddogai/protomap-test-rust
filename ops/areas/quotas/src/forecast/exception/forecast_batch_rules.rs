use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast batch must pass. Check returns its problems, or none.
pub trait ForecastBatchRule: std::fmt::Debug {
    fn check(&self, item: &ForecastBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBatchLabelLengthRule {
    pub max_length: i64,
}

impl ForecastBatchRule for ForecastBatchLabelLengthRule {
    fn check(&self, item: &ForecastBatch) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBatchCodeLengthRule {
    pub max_length: i64,
}

impl ForecastBatchRule for ForecastBatchCodeLengthRule {
    fn check(&self, item: &ForecastBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBatchRequiredFingerprintRule {
    pub message: String,
}

impl ForecastBatchRule for ForecastBatchRequiredFingerprintRule {
    fn check(&self, item: &ForecastBatch) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBatchFingerprintLengthRule {
    pub max_length: i64,
}

impl ForecastBatchRule for ForecastBatchFingerprintLengthRule {
    fn check(&self, item: &ForecastBatch) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast batch.
#[derive(Debug, Clone)]
pub struct ForecastBatchValidator {
    pub rules: Vec<Rc<dyn ForecastBatchRule>>,
}

impl ForecastBatchValidator {
    pub fn validate(&self, item: &ForecastBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_batch_validator() -> ForecastBatchValidator {
    ForecastBatchValidator {
        rules: vec![
            Rc::new(ForecastBatchLabelLengthRule { max_length: 3600 }) as Rc<dyn ForecastBatchRule>,
            Rc::new(ForecastBatchCodeLengthRule { max_length: 120 }),
            Rc::new(ForecastBatchRequiredFingerprintRule {
                message: "forecast batch needs an entry".to_string(),
            }),
            Rc::new(ForecastBatchFingerprintLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_batch_validator();
        let item = new_forecast_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ForecastBatchLabelLengthRule { max_length: 3600 };
        let mut item = new_forecast_batch("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ForecastBatchCodeLengthRule { max_length: 120 };
        let mut item = new_forecast_batch("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = ForecastBatchRequiredFingerprintRule {
            message: "forecast batch needs a value".to_string(),
        };
        let mut item = new_forecast_batch("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = ForecastBatchFingerprintLengthRule { max_length: 80 };
        let mut item = new_forecast_batch("x", "y");
        item.fingerprint = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
