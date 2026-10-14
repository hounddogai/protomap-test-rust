use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback reservation forecast must pass. Check returns its problems, or none.
pub trait FallbackReservationForecastRule: std::fmt::Debug {
    fn check(&self, item: &FallbackReservationForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationForecastUnitLengthRule {
    pub max_length: i64,
}

impl FallbackReservationForecastRule for FallbackReservationForecastUnitLengthRule {
    fn check(&self, item: &FallbackReservationForecast) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationForecastCipherLengthRule {
    pub max_length: i64,
}

impl FallbackReservationForecastRule for FallbackReservationForecastCipherLengthRule {
    fn check(&self, item: &FallbackReservationForecast) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationForecastTitleLengthRule {
    pub max_length: i64,
}

impl FallbackReservationForecastRule for FallbackReservationForecastTitleLengthRule {
    fn check(&self, item: &FallbackReservationForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback reservation forecast.
#[derive(Debug, Clone)]
pub struct FallbackReservationForecastValidator {
    pub rules: Vec<Rc<dyn FallbackReservationForecastRule>>,
}

impl FallbackReservationForecastValidator {
    pub fn validate(&self, item: &FallbackReservationForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackReservationForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackReservationForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_reservation_forecast_validator() -> FallbackReservationForecastValidator {
    FallbackReservationForecastValidator {
        rules: vec![
            Rc::new(FallbackReservationForecastUnitLengthRule { max_length: 255 })
                as Rc<dyn FallbackReservationForecastRule>,
            Rc::new(FallbackReservationForecastCipherLengthRule { max_length: 3600 }),
            Rc::new(FallbackReservationForecastTitleLengthRule { max_length: 32 }),
        ],
    }
}
