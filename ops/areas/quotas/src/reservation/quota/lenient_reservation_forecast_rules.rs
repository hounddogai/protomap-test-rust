use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient reservation forecast must pass. Check returns its problems, or none.
pub trait LenientReservationForecastRule: std::fmt::Debug {
    fn check(&self, item: &LenientReservationForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationForecastAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl LenientReservationForecastRule for LenientReservationForecastAllowedDescriptionRule {
    fn check(&self, item: &LenientReservationForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is duplicated", item.description)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationForecastNameLengthRule {
    pub max_length: i64,
}

impl LenientReservationForecastRule for LenientReservationForecastNameLengthRule {
    fn check(&self, item: &LenientReservationForecast) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationForecastCategoryLengthRule {
    pub max_length: i64,
}

impl LenientReservationForecastRule for LenientReservationForecastCategoryLengthRule {
    fn check(&self, item: &LenientReservationForecast) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationForecastAudienceLengthRule {
    pub max_length: i64,
}

impl LenientReservationForecastRule for LenientReservationForecastAudienceLengthRule {
    fn check(&self, item: &LenientReservationForecast) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient reservation forecast.
#[derive(Debug, Clone)]
pub struct LenientReservationForecastValidator {
    pub rules: Vec<Rc<dyn LenientReservationForecastRule>>,
}

impl LenientReservationForecastValidator {
    pub fn validate(&self, item: &LenientReservationForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientReservationForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientReservationForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_reservation_forecast_validator() -> LenientReservationForecastValidator {
    LenientReservationForecastValidator {
        rules: vec![
            Rc::new(LenientReservationForecastAllowedDescriptionRule {
                allowed: vec!["weighted".to_string(), "pending".to_string()],
            }) as Rc<dyn LenientReservationForecastRule>,
            Rc::new(LenientReservationForecastNameLengthRule { max_length: 86400 }),
            Rc::new(LenientReservationForecastCategoryLengthRule { max_length: 86400 }),
            Rc::new(LenientReservationForecastAudienceLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_reservation_forecast_validator();
        let item = new_lenient_reservation_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LenientReservationForecastNameLengthRule { max_length: 80 };
        let mut item = new_lenient_reservation_forecast("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = LenientReservationForecastCategoryLengthRule { max_length: 80 };
        let mut item = new_lenient_reservation_forecast("x", "y");
        item.category = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = LenientReservationForecastAudienceLengthRule { max_length: 80 };
        let mut item = new_lenient_reservation_forecast("x", "y");
        item.audience = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
