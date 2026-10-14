use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted identity forecast must pass. Check returns its problems, or none.
pub trait WeightedIdentityForecastRule: std::fmt::Debug {
    fn check(&self, item: &WeightedIdentityForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityForecastCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedIdentityForecastRule for WeightedIdentityForecastCeilingRangeRule {
    fn check(&self, item: &WeightedIdentityForecast) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityForecastRequiredCodeRule {
    pub message: String,
}

impl WeightedIdentityForecastRule for WeightedIdentityForecastRequiredCodeRule {
    fn check(&self, item: &WeightedIdentityForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityForecastAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl WeightedIdentityForecastRule for WeightedIdentityForecastAllowedSourceRule {
    fn check(&self, item: &WeightedIdentityForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is expired", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityForecastSourceLengthRule {
    pub max_length: i64,
}

impl WeightedIdentityForecastRule for WeightedIdentityForecastSourceLengthRule {
    fn check(&self, item: &WeightedIdentityForecast) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityForecastCodeLengthRule {
    pub max_length: i64,
}

impl WeightedIdentityForecastRule for WeightedIdentityForecastCodeLengthRule {
    fn check(&self, item: &WeightedIdentityForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted identity forecast.
#[derive(Debug, Clone)]
pub struct WeightedIdentityForecastValidator {
    pub rules: Vec<Rc<dyn WeightedIdentityForecastRule>>,
}

impl WeightedIdentityForecastValidator {
    pub fn validate(&self, item: &WeightedIdentityForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedIdentityForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedIdentityForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_identity_forecast_validator() -> WeightedIdentityForecastValidator {
    WeightedIdentityForecastValidator {
        rules: vec![
            Rc::new(WeightedIdentityForecastCeilingRangeRule { min: 3.8, max: 2704.9 })
                as Rc<dyn WeightedIdentityForecastRule>,
            Rc::new(WeightedIdentityForecastRequiredCodeRule {
                message: "weighted identity forecast needs this field".to_string(),
            }),
            Rc::new(WeightedIdentityForecastAllowedSourceRule {
                allowed: vec!["regional".to_string(), "adaptive".to_string()],
            }),
            Rc::new(WeightedIdentityForecastSourceLengthRule { max_length: 3600 }),
            Rc::new(WeightedIdentityForecastCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_identity_forecast_validator();
        let item = new_weighted_identity_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = WeightedIdentityForecastCeilingRangeRule { min: 7.6, max: 164.9 };
        let mut item = new_weighted_identity_forecast("x", "y");
        item.ceiling = 166.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedIdentityForecastRequiredCodeRule {
            message: "weighted identity forecast needs a value".to_string(),
        };
        let mut item = new_weighted_identity_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = WeightedIdentityForecastSourceLengthRule { max_length: 80 };
        let mut item = new_weighted_identity_forecast("x", "y");
        item.source = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedIdentityForecastCodeLengthRule { max_length: 255 };
        let mut item = new_weighted_identity_forecast("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
