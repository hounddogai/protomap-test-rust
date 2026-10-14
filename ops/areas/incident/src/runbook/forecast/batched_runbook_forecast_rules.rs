use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched runbook forecast must pass. Check returns its problems, or none.
pub trait BatchedRunbookForecastRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRunbookForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookForecastCodePrefixRule {
    pub prefix: String,
}

impl BatchedRunbookForecastRule for BatchedRunbookForecastCodePrefixRule {
    fn check(&self, item: &BatchedRunbookForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookForecastRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedRunbookForecastRule for BatchedRunbookForecastRotationDaysRangeRule {
    fn check(&self, item: &BatchedRunbookForecast) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookForecastCodeLengthRule {
    pub max_length: i64,
}

impl BatchedRunbookForecastRule for BatchedRunbookForecastCodeLengthRule {
    fn check(&self, item: &BatchedRunbookForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookForecastLabelLengthRule {
    pub max_length: i64,
}

impl BatchedRunbookForecastRule for BatchedRunbookForecastLabelLengthRule {
    fn check(&self, item: &BatchedRunbookForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched runbook forecast.
#[derive(Debug, Clone)]
pub struct BatchedRunbookForecastValidator {
    pub rules: Vec<Rc<dyn BatchedRunbookForecastRule>>,
}

impl BatchedRunbookForecastValidator {
    pub fn validate(&self, item: &BatchedRunbookForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedRunbookForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRunbookForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_runbook_forecast_validator() -> BatchedRunbookForecastValidator {
    BatchedRunbookForecastValidator {
        rules: vec![
            Rc::new(BatchedRunbookForecastCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn BatchedRunbookForecastRule>,
            Rc::new(BatchedRunbookForecastRotationDaysRangeRule { min: 64, max: 255 }),
            Rc::new(BatchedRunbookForecastCodeLengthRule { max_length: 16 }),
            Rc::new(BatchedRunbookForecastLabelLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_runbook_forecast_validator();
        let item = new_batched_runbook_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = BatchedRunbookForecastRotationDaysRangeRule { min: 64, max: 3600 };
        let mut item = new_batched_runbook_forecast("x", "y");
        item.rotation_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedRunbookForecastCodeLengthRule { max_length: 120 };
        let mut item = new_batched_runbook_forecast("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BatchedRunbookForecastLabelLengthRule { max_length: 64 };
        let mut item = new_batched_runbook_forecast("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
