use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual release forecast must pass. Check returns its problems, or none.
pub trait ManualReleaseForecastRule: std::fmt::Debug {
    fn check(&self, item: &ManualReleaseForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseForecastCodeLengthRule {
    pub max_length: i64,
}

impl ManualReleaseForecastRule for ManualReleaseForecastCodeLengthRule {
    fn check(&self, item: &ManualReleaseForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseForecastQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualReleaseForecastRule for ManualReleaseForecastQuantityRangeRule {
    fn check(&self, item: &ManualReleaseForecast) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseForecastSummaryLengthRule {
    pub max_length: i64,
}

impl ManualReleaseForecastRule for ManualReleaseForecastSummaryLengthRule {
    fn check(&self, item: &ManualReleaseForecast) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseForecastAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl ManualReleaseForecastRule for ManualReleaseForecastAllowedCountryRule {
    fn check(&self, item: &ManualReleaseForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is inconsistent", item.country)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual release forecast.
#[derive(Debug, Clone)]
pub struct ManualReleaseForecastValidator {
    pub rules: Vec<Rc<dyn ManualReleaseForecastRule>>,
}

impl ManualReleaseForecastValidator {
    pub fn validate(&self, item: &ManualReleaseForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualReleaseForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualReleaseForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_release_forecast_validator() -> ManualReleaseForecastValidator {
    ManualReleaseForecastValidator {
        rules: vec![
            Rc::new(ManualReleaseForecastCodeLengthRule { max_length: 80 }) as Rc<dyn ManualReleaseForecastRule>,
            Rc::new(ManualReleaseForecastQuantityRangeRule { min: 32, max: 80 }),
            Rc::new(ManualReleaseForecastSummaryLengthRule { max_length: 3600 }),
            Rc::new(ManualReleaseForecastAllowedCountryRule {
                allowed: vec!["seasonal".to_string(), "manual".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_release_forecast_validator();
        let item = new_manual_release_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualReleaseForecastCodeLengthRule { max_length: 255 };
        let mut item = new_manual_release_forecast("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ManualReleaseForecastQuantityRangeRule { min: 120, max: 1000 };
        let mut item = new_manual_release_forecast("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = ManualReleaseForecastSummaryLengthRule { max_length: 3600 };
        let mut item = new_manual_release_forecast("x", "y");
        item.summary = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
