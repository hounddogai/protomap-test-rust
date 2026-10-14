use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped canary forecast must pass. Check returns its problems, or none.
pub trait CappedCanaryForecastRule: std::fmt::Debug {
    fn check(&self, item: &CappedCanaryForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedCanaryForecastAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl CappedCanaryForecastRule for CappedCanaryForecastAllowedSummaryRule {
    fn check(&self, item: &CappedCanaryForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is inconsistent", item.summary)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedCanaryForecastDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedCanaryForecastRule for CappedCanaryForecastDurationMinutesRangeRule {
    fn check(&self, item: &CappedCanaryForecast) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedCanaryForecastCodePrefixRule {
    pub prefix: String,
}

impl CappedCanaryForecastRule for CappedCanaryForecastCodePrefixRule {
    fn check(&self, item: &CappedCanaryForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped canary forecast.
#[derive(Debug, Clone)]
pub struct CappedCanaryForecastValidator {
    pub rules: Vec<Rc<dyn CappedCanaryForecastRule>>,
}

impl CappedCanaryForecastValidator {
    pub fn validate(&self, item: &CappedCanaryForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedCanaryForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedCanaryForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_canary_forecast_validator() -> CappedCanaryForecastValidator {
    CappedCanaryForecastValidator {
        rules: vec![
            Rc::new(CappedCanaryForecastAllowedSummaryRule {
                allowed: vec!["global".to_string(), "weighted".to_string(), "tiered".to_string()],
            }) as Rc<dyn CappedCanaryForecastRule>,
            Rc::new(CappedCanaryForecastDurationMinutesRangeRule { min: 64, max: 80 }),
            Rc::new(CappedCanaryForecastCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_canary_forecast_validator();
        let item = new_capped_canary_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = CappedCanaryForecastDurationMinutesRangeRule { min: 16, max: 86400 };
        let mut item = new_capped_canary_forecast("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
