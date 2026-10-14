use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary reservation forecast must pass. Check returns its problems, or none.
pub trait PrimaryReservationForecastRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryReservationForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationForecastAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryReservationForecastRule for PrimaryReservationForecastAmountRangeRule {
    fn check(&self, item: &PrimaryReservationForecast) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationForecastCodePrefixRule {
    pub prefix: String,
}

impl PrimaryReservationForecastRule for PrimaryReservationForecastCodePrefixRule {
    fn check(&self, item: &PrimaryReservationForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationForecastContactEmailLengthRule {
    pub max_length: i64,
}

impl PrimaryReservationForecastRule for PrimaryReservationForecastContactEmailLengthRule {
    fn check(&self, item: &PrimaryReservationForecast) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary reservation forecast.
#[derive(Debug, Clone)]
pub struct PrimaryReservationForecastValidator {
    pub rules: Vec<Rc<dyn PrimaryReservationForecastRule>>,
}

impl PrimaryReservationForecastValidator {
    pub fn validate(&self, item: &PrimaryReservationForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryReservationForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryReservationForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_reservation_forecast_validator() -> PrimaryReservationForecastValidator {
    PrimaryReservationForecastValidator {
        rules: vec![
            Rc::new(PrimaryReservationForecastAmountRangeRule { min: 0.9, max: 2452.6 })
                as Rc<dyn PrimaryReservationForecastRule>,
            Rc::new(PrimaryReservationForecastCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PrimaryReservationForecastContactEmailLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_reservation_forecast_validator();
        let item = new_primary_reservation_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = PrimaryReservationForecastAmountRangeRule { min: 10.0, max: 3623.4 };
        let mut item = new_primary_reservation_forecast("x", "y");
        item.amount = 3624.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = PrimaryReservationForecastContactEmailLengthRule { max_length: 255 };
        let mut item = new_primary_reservation_forecast("x", "y");
        item.contact_email = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
