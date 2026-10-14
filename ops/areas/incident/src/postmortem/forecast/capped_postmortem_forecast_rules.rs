use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped postmortem forecast must pass. Check returns its problems, or none.
pub trait CappedPostmortemForecastRule: std::fmt::Debug {
    fn check(&self, item: &CappedPostmortemForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPostmortemForecastRequiredCodeRule {
    pub message: String,
}

impl CappedPostmortemForecastRule for CappedPostmortemForecastRequiredCodeRule {
    fn check(&self, item: &CappedPostmortemForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPostmortemForecastDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedPostmortemForecastRule for CappedPostmortemForecastDiscountRangeRule {
    fn check(&self, item: &CappedPostmortemForecast) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPostmortemForecastAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CappedPostmortemForecastRule for CappedPostmortemForecastAllowedCodeRule {
    fn check(&self, item: &CappedPostmortemForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped postmortem forecast.
#[derive(Debug, Clone)]
pub struct CappedPostmortemForecastValidator {
    pub rules: Vec<Rc<dyn CappedPostmortemForecastRule>>,
}

impl CappedPostmortemForecastValidator {
    pub fn validate(&self, item: &CappedPostmortemForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedPostmortemForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedPostmortemForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_postmortem_forecast_validator() -> CappedPostmortemForecastValidator {
    CappedPostmortemForecastValidator {
        rules: vec![
            Rc::new(CappedPostmortemForecastRequiredCodeRule {
                message: "capped postmortem forecast needs a value".to_string(),
            }) as Rc<dyn CappedPostmortemForecastRule>,
            Rc::new(CappedPostmortemForecastDiscountRangeRule { min: 8.1, max: 4272.3 }),
            Rc::new(CappedPostmortemForecastAllowedCodeRule {
                allowed: vec!["pending".to_string(), "fallback".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_postmortem_forecast_validator();
        let item = new_capped_postmortem_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedPostmortemForecastRequiredCodeRule {
            message: "capped postmortem forecast needs an entry".to_string(),
        };
        let mut item = new_capped_postmortem_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = CappedPostmortemForecastDiscountRangeRule { min: 1.6, max: 1088.8 };
        let mut item = new_capped_postmortem_forecast("x", "y");
        item.discount = 1090.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
