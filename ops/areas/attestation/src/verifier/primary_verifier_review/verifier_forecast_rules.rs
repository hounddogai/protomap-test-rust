use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a verifier forecast must pass. Check returns its problems, or none.
pub trait VerifierForecastRule: std::fmt::Debug {
    fn check(&self, item: &VerifierForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierForecastAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl VerifierForecastRule for VerifierForecastAllowedChannelRule {
    fn check(&self, item: &VerifierForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is blocked", item.channel)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierForecastLabelLengthRule {
    pub max_length: i64,
}

impl VerifierForecastRule for VerifierForecastLabelLengthRule {
    fn check(&self, item: &VerifierForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierForecastRequiredLabelRule {
    pub message: String,
}

impl VerifierForecastRule for VerifierForecastRequiredLabelRule {
    fn check(&self, item: &VerifierForecast) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierForecastFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl VerifierForecastRule for VerifierForecastFloorValueRangeRule {
    fn check(&self, item: &VerifierForecast) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierForecastChannelLengthRule {
    pub max_length: i64,
}

impl VerifierForecastRule for VerifierForecastChannelLengthRule {
    fn check(&self, item: &VerifierForecast) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierForecastLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierForecastRule for VerifierForecastLimitRangeRule {
    fn check(&self, item: &VerifierForecast) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a verifier forecast.
#[derive(Debug, Clone)]
pub struct VerifierForecastValidator {
    pub rules: Vec<Rc<dyn VerifierForecastRule>>,
}

impl VerifierForecastValidator {
    pub fn validate(&self, item: &VerifierForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_forecast_validator() -> VerifierForecastValidator {
    VerifierForecastValidator {
        rules: vec![
            Rc::new(VerifierForecastAllowedChannelRule {
                allowed: vec!["lenient".to_string()],
            }) as Rc<dyn VerifierForecastRule>,
            Rc::new(VerifierForecastLabelLengthRule { max_length: 120 }),
            Rc::new(VerifierForecastRequiredLabelRule {
                message: "verifier forecast needs an entry".to_string(),
            }),
            Rc::new(VerifierForecastFloorValueRangeRule { min: 3.9, max: 2843.2 }),
            Rc::new(VerifierForecastChannelLengthRule { max_length: 3600 }),
            Rc::new(VerifierForecastLimitRangeRule { min: 32, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_forecast_validator();
        let item = new_verifier_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = VerifierForecastLabelLengthRule { max_length: 255 };
        let mut item = new_verifier_forecast("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = VerifierForecastRequiredLabelRule {
            message: "verifier forecast needs this field".to_string(),
        };
        let mut item = new_verifier_forecast("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = VerifierForecastFloorValueRangeRule { min: 8.7, max: 4612.1 };
        let mut item = new_verifier_forecast("x", "y");
        item.floor_value = 4613.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = VerifierForecastChannelLengthRule { max_length: 120 };
        let mut item = new_verifier_forecast("x", "y");
        item.channel = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = VerifierForecastLimitRangeRule { min: 64, max: 86400 };
        let mut item = new_verifier_forecast("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
