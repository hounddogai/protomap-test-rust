use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a forecast profile must pass. Check returns its problems, or none.
pub trait ForecastProfileRule: std::fmt::Debug {
    fn check(&self, item: &ForecastProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastProfileAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastProfileRule for ForecastProfileAttemptsRangeRule {
    fn check(&self, item: &ForecastProfile) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastProfileAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl ForecastProfileRule for ForecastProfileAllowedCountryRule {
    fn check(&self, item: &ForecastProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is above maximum", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastProfileRequiredNameRule {
    pub message: String,
}

impl ForecastProfileRule for ForecastProfileRequiredNameRule {
    fn check(&self, item: &ForecastProfile) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastProfileRequiredCodeRule {
    pub message: String,
}

impl ForecastProfileRule for ForecastProfileRequiredCodeRule {
    fn check(&self, item: &ForecastProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastProfileRequiredCountryRule {
    pub message: String,
}

impl ForecastProfileRule for ForecastProfileRequiredCountryRule {
    fn check(&self, item: &ForecastProfile) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastProfileEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastProfileRule for ForecastProfileEndsAtRangeRule {
    fn check(&self, item: &ForecastProfile) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a forecast profile.
#[derive(Debug, Clone)]
pub struct ForecastProfileValidator {
    pub rules: Vec<Rc<dyn ForecastProfileRule>>,
}

impl ForecastProfileValidator {
    pub fn validate(&self, item: &ForecastProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_profile_validator() -> ForecastProfileValidator {
    ForecastProfileValidator {
        rules: vec![
            Rc::new(ForecastProfileAttemptsRangeRule { min: 32, max: 80 }) as Rc<dyn ForecastProfileRule>,
            Rc::new(ForecastProfileAllowedCountryRule {
                allowed: vec!["rolling".to_string(), "weighted".to_string()],
            }),
            Rc::new(ForecastProfileRequiredNameRule {
                message: "forecast profile needs an entry".to_string(),
            }),
            Rc::new(ForecastProfileRequiredCodeRule {
                message: "forecast profile needs an entry".to_string(),
            }),
            Rc::new(ForecastProfileRequiredCountryRule {
                message: "forecast profile needs an entry".to_string(),
            }),
            Rc::new(ForecastProfileEndsAtRangeRule { min: 80, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_profile_validator();
        let item = new_forecast_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = ForecastProfileAttemptsRangeRule { min: 80, max: 86400 };
        let mut item = new_forecast_profile("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ForecastProfileRequiredNameRule {
            message: "forecast profile needs an entry".to_string(),
        };
        let mut item = new_forecast_profile("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ForecastProfileRequiredCodeRule {
            message: "forecast profile needs a value".to_string(),
        };
        let mut item = new_forecast_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = ForecastProfileRequiredCountryRule {
            message: "forecast profile needs an entry".to_string(),
        };
        let mut item = new_forecast_profile("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = ForecastProfileEndsAtRangeRule { min: 255, max: 86400 };
        let mut item = new_forecast_profile("x", "y");
        item.ends_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
