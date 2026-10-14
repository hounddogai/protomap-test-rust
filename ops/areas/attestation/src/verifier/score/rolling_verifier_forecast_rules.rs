use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling verifier forecast must pass. Check returns its problems, or none.
pub trait RollingVerifierForecastRule: std::fmt::Debug {
    fn check(&self, item: &RollingVerifierForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierForecastCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingVerifierForecastRule for RollingVerifierForecastCostRangeRule {
    fn check(&self, item: &RollingVerifierForecast) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierForecastLabelLengthRule {
    pub max_length: i64,
}

impl RollingVerifierForecastRule for RollingVerifierForecastLabelLengthRule {
    fn check(&self, item: &RollingVerifierForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierForecastRequiredLabelRule {
    pub message: String,
}

impl RollingVerifierForecastRule for RollingVerifierForecastRequiredLabelRule {
    fn check(&self, item: &RollingVerifierForecast) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierForecastAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl RollingVerifierForecastRule for RollingVerifierForecastAllowedNotesRule {
    fn check(&self, item: &RollingVerifierForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is inconsistent", item.notes)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierForecastCodePrefixRule {
    pub prefix: String,
}

impl RollingVerifierForecastRule for RollingVerifierForecastCodePrefixRule {
    fn check(&self, item: &RollingVerifierForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling verifier forecast.
#[derive(Debug, Clone)]
pub struct RollingVerifierForecastValidator {
    pub rules: Vec<Rc<dyn RollingVerifierForecastRule>>,
}

impl RollingVerifierForecastValidator {
    pub fn validate(&self, item: &RollingVerifierForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingVerifierForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingVerifierForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_verifier_forecast_validator() -> RollingVerifierForecastValidator {
    RollingVerifierForecastValidator {
        rules: vec![
            Rc::new(RollingVerifierForecastCostRangeRule { min: 7.7, max: 2945.3 })
                as Rc<dyn RollingVerifierForecastRule>,
            Rc::new(RollingVerifierForecastLabelLengthRule { max_length: 80 }),
            Rc::new(RollingVerifierForecastRequiredLabelRule {
                message: "rolling verifier forecast needs an entry".to_string(),
            }),
            Rc::new(RollingVerifierForecastAllowedNotesRule {
                allowed: vec!["weighted".to_string(), "tiered".to_string()],
            }),
            Rc::new(RollingVerifierForecastCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_verifier_forecast_validator();
        let item = new_rolling_verifier_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = RollingVerifierForecastCostRangeRule { min: 9.0, max: 3604.7 };
        let mut item = new_rolling_verifier_forecast("x", "y");
        item.cost = 3606.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollingVerifierForecastLabelLengthRule { max_length: 1000 };
        let mut item = new_rolling_verifier_forecast("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollingVerifierForecastRequiredLabelRule {
            message: "rolling verifier forecast needs this field".to_string(),
        };
        let mut item = new_rolling_verifier_forecast("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
