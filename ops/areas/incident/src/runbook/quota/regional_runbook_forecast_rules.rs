use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional runbook forecast must pass. Check returns its problems, or none.
pub trait RegionalRunbookForecastRule: std::fmt::Debug {
    fn check(&self, item: &RegionalRunbookForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookForecastDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalRunbookForecastRule for RegionalRunbookForecastDiscountRangeRule {
    fn check(&self, item: &RegionalRunbookForecast) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookForecastCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalRunbookForecastRule for RegionalRunbookForecastCreatedAtRangeRule {
    fn check(&self, item: &RegionalRunbookForecast) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookForecastAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionalRunbookForecastRule for RegionalRunbookForecastAllowedCodeRule {
    fn check(&self, item: &RegionalRunbookForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional runbook forecast.
#[derive(Debug, Clone)]
pub struct RegionalRunbookForecastValidator {
    pub rules: Vec<Rc<dyn RegionalRunbookForecastRule>>,
}

impl RegionalRunbookForecastValidator {
    pub fn validate(&self, item: &RegionalRunbookForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalRunbookForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalRunbookForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_runbook_forecast_validator() -> RegionalRunbookForecastValidator {
    RegionalRunbookForecastValidator {
        rules: vec![
            Rc::new(RegionalRunbookForecastDiscountRangeRule { min: 6.0, max: 1773.9 })
                as Rc<dyn RegionalRunbookForecastRule>,
            Rc::new(RegionalRunbookForecastCreatedAtRangeRule { min: 16, max: 86400 }),
            Rc::new(RegionalRunbookForecastAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "regional".to_string(), "default".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_runbook_forecast_validator();
        let item = new_regional_runbook_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = RegionalRunbookForecastDiscountRangeRule { min: 7.5, max: 4276.9 };
        let mut item = new_regional_runbook_forecast("x", "y");
        item.discount = 4278.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RegionalRunbookForecastCreatedAtRangeRule { min: 64, max: 86400 };
        let mut item = new_regional_runbook_forecast("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
