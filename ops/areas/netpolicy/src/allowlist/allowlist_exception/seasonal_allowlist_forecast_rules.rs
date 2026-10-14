use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal allowlist forecast must pass. Check returns its problems, or none.
pub trait SeasonalAllowlistForecastRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalAllowlistForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAllowlistForecastAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl SeasonalAllowlistForecastRule for SeasonalAllowlistForecastAllowedSummaryRule {
    fn check(&self, item: &SeasonalAllowlistForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is below minimum", item.summary)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAllowlistForecastRequiredCountryRule {
    pub message: String,
}

impl SeasonalAllowlistForecastRule for SeasonalAllowlistForecastRequiredCountryRule {
    fn check(&self, item: &SeasonalAllowlistForecast) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAllowlistForecastRequiredCodeRule {
    pub message: String,
}

impl SeasonalAllowlistForecastRule for SeasonalAllowlistForecastRequiredCodeRule {
    fn check(&self, item: &SeasonalAllowlistForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAllowlistForecastScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalAllowlistForecastRule for SeasonalAllowlistForecastScoreRangeRule {
    fn check(&self, item: &SeasonalAllowlistForecast) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAllowlistForecastRequiredSummaryRule {
    pub message: String,
}

impl SeasonalAllowlistForecastRule for SeasonalAllowlistForecastRequiredSummaryRule {
    fn check(&self, item: &SeasonalAllowlistForecast) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAllowlistForecastSummaryLengthRule {
    pub max_length: i64,
}

impl SeasonalAllowlistForecastRule for SeasonalAllowlistForecastSummaryLengthRule {
    fn check(&self, item: &SeasonalAllowlistForecast) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal allowlist forecast.
#[derive(Debug, Clone)]
pub struct SeasonalAllowlistForecastValidator {
    pub rules: Vec<Rc<dyn SeasonalAllowlistForecastRule>>,
}

impl SeasonalAllowlistForecastValidator {
    pub fn validate(&self, item: &SeasonalAllowlistForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalAllowlistForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalAllowlistForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_allowlist_forecast_validator() -> SeasonalAllowlistForecastValidator {
    SeasonalAllowlistForecastValidator {
        rules: vec![
            Rc::new(SeasonalAllowlistForecastAllowedSummaryRule { allowed: Vec::new() })
                as Rc<dyn SeasonalAllowlistForecastRule>,
            Rc::new(SeasonalAllowlistForecastRequiredCountryRule {
                message: "seasonal allowlist forecast needs this field".to_string(),
            }),
            Rc::new(SeasonalAllowlistForecastRequiredCodeRule {
                message: "seasonal allowlist forecast needs this field".to_string(),
            }),
            Rc::new(SeasonalAllowlistForecastScoreRangeRule { min: 1.6, max: 552.4 }),
            Rc::new(SeasonalAllowlistForecastRequiredSummaryRule {
                message: "seasonal allowlist forecast needs a value".to_string(),
            }),
            Rc::new(SeasonalAllowlistForecastSummaryLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_allowlist_forecast_validator();
        let item = new_seasonal_allowlist_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = SeasonalAllowlistForecastRequiredCountryRule {
            message: "seasonal allowlist forecast needs an entry".to_string(),
        };
        let mut item = new_seasonal_allowlist_forecast("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalAllowlistForecastRequiredCodeRule {
            message: "seasonal allowlist forecast needs a value".to_string(),
        };
        let mut item = new_seasonal_allowlist_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = SeasonalAllowlistForecastScoreRangeRule { min: 5.2, max: 2762.6 };
        let mut item = new_seasonal_allowlist_forecast("x", "y");
        item.score = 2764.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = SeasonalAllowlistForecastRequiredSummaryRule {
            message: "seasonal allowlist forecast needs this field".to_string(),
        };
        let mut item = new_seasonal_allowlist_forecast("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = SeasonalAllowlistForecastSummaryLengthRule { max_length: 64 };
        let mut item = new_seasonal_allowlist_forecast("x", "y");
        item.summary = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
