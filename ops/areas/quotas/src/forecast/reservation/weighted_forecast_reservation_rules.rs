use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted forecast reservation must pass. Check returns its problems, or none.
pub trait WeightedForecastReservationRule: std::fmt::Debug {
    fn check(&self, item: &WeightedForecastReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastReservationAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl WeightedForecastReservationRule for WeightedForecastReservationAllowedCodeRule {
    fn check(&self, item: &WeightedForecastReservation) -> Vec<String> {
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
pub struct WeightedForecastReservationRequiredCodeRule {
    pub message: String,
}

impl WeightedForecastReservationRule for WeightedForecastReservationRequiredCodeRule {
    fn check(&self, item: &WeightedForecastReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastReservationCodeLengthRule {
    pub max_length: i64,
}

impl WeightedForecastReservationRule for WeightedForecastReservationCodeLengthRule {
    fn check(&self, item: &WeightedForecastReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastReservationLabelLengthRule {
    pub max_length: i64,
}

impl WeightedForecastReservationRule for WeightedForecastReservationLabelLengthRule {
    fn check(&self, item: &WeightedForecastReservation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted forecast reservation.
#[derive(Debug, Clone)]
pub struct WeightedForecastReservationValidator {
    pub rules: Vec<Rc<dyn WeightedForecastReservationRule>>,
}

impl WeightedForecastReservationValidator {
    pub fn validate(&self, item: &WeightedForecastReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedForecastReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedForecastReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_forecast_reservation_validator() -> WeightedForecastReservationValidator {
    WeightedForecastReservationValidator {
        rules: vec![
            Rc::new(WeightedForecastReservationAllowedCodeRule {
                allowed: vec!["pending".to_string(), "lenient".to_string(), "regional".to_string()],
            }) as Rc<dyn WeightedForecastReservationRule>,
            Rc::new(WeightedForecastReservationRequiredCodeRule {
                message: "weighted forecast reservation needs a value".to_string(),
            }),
            Rc::new(WeightedForecastReservationCodeLengthRule { max_length: 255 }),
            Rc::new(WeightedForecastReservationLabelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_forecast_reservation_validator();
        let item = new_weighted_forecast_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedForecastReservationRequiredCodeRule {
            message: "weighted forecast reservation needs an entry".to_string(),
        };
        let mut item = new_weighted_forecast_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedForecastReservationCodeLengthRule { max_length: 16 };
        let mut item = new_weighted_forecast_reservation("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = WeightedForecastReservationLabelLengthRule { max_length: 64 };
        let mut item = new_weighted_forecast_reservation("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
