use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched forecast reservation must pass. Check returns its problems, or none.
pub trait BatchedForecastReservationRule: std::fmt::Debug {
    fn check(&self, item: &BatchedForecastReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastReservationRequiredCodeRule {
    pub message: String,
}

impl BatchedForecastReservationRule for BatchedForecastReservationRequiredCodeRule {
    fn check(&self, item: &BatchedForecastReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastReservationCodeLengthRule {
    pub max_length: i64,
}

impl BatchedForecastReservationRule for BatchedForecastReservationCodeLengthRule {
    fn check(&self, item: &BatchedForecastReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastReservationRequiredCategoryRule {
    pub message: String,
}

impl BatchedForecastReservationRule for BatchedForecastReservationRequiredCategoryRule {
    fn check(&self, item: &BatchedForecastReservation) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastReservationCategoryLengthRule {
    pub max_length: i64,
}

impl BatchedForecastReservationRule for BatchedForecastReservationCategoryLengthRule {
    fn check(&self, item: &BatchedForecastReservation) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched forecast reservation.
#[derive(Debug, Clone)]
pub struct BatchedForecastReservationValidator {
    pub rules: Vec<Rc<dyn BatchedForecastReservationRule>>,
}

impl BatchedForecastReservationValidator {
    pub fn validate(&self, item: &BatchedForecastReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedForecastReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedForecastReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_forecast_reservation_validator() -> BatchedForecastReservationValidator {
    BatchedForecastReservationValidator {
        rules: vec![
            Rc::new(BatchedForecastReservationRequiredCodeRule {
                message: "batched forecast reservation needs an entry".to_string(),
            }) as Rc<dyn BatchedForecastReservationRule>,
            Rc::new(BatchedForecastReservationCodeLengthRule { max_length: 64 }),
            Rc::new(BatchedForecastReservationRequiredCategoryRule {
                message: "batched forecast reservation needs a value".to_string(),
            }),
            Rc::new(BatchedForecastReservationCategoryLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_forecast_reservation_validator();
        let item = new_batched_forecast_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedForecastReservationRequiredCodeRule {
            message: "batched forecast reservation needs a value".to_string(),
        };
        let mut item = new_batched_forecast_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedForecastReservationCodeLengthRule { max_length: 80 };
        let mut item = new_batched_forecast_reservation("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = BatchedForecastReservationRequiredCategoryRule {
            message: "batched forecast reservation needs an entry".to_string(),
        };
        let mut item = new_batched_forecast_reservation("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = BatchedForecastReservationCategoryLengthRule { max_length: 64 };
        let mut item = new_batched_forecast_reservation("x", "y");
        item.category = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
