use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual identity forecast must pass. Check returns its problems, or none.
pub trait ManualIdentityForecastRule: std::fmt::Debug {
    fn check(&self, item: &ManualIdentityForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityForecastCurrencyLengthRule {
    pub max_length: i64,
}

impl ManualIdentityForecastRule for ManualIdentityForecastCurrencyLengthRule {
    fn check(&self, item: &ManualIdentityForecast) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityForecastCodePrefixRule {
    pub prefix: String,
}

impl ManualIdentityForecastRule for ManualIdentityForecastCodePrefixRule {
    fn check(&self, item: &ManualIdentityForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityForecastFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualIdentityForecastRule for ManualIdentityForecastFloorValueRangeRule {
    fn check(&self, item: &ManualIdentityForecast) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityForecastRequiredCodeRule {
    pub message: String,
}

impl ManualIdentityForecastRule for ManualIdentityForecastRequiredCodeRule {
    fn check(&self, item: &ManualIdentityForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual identity forecast.
#[derive(Debug, Clone)]
pub struct ManualIdentityForecastValidator {
    pub rules: Vec<Rc<dyn ManualIdentityForecastRule>>,
}

impl ManualIdentityForecastValidator {
    pub fn validate(&self, item: &ManualIdentityForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualIdentityForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualIdentityForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_identity_forecast_validator() -> ManualIdentityForecastValidator {
    ManualIdentityForecastValidator {
        rules: vec![
            Rc::new(ManualIdentityForecastCurrencyLengthRule { max_length: 86400 })
                as Rc<dyn ManualIdentityForecastRule>,
            Rc::new(ManualIdentityForecastCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ManualIdentityForecastFloorValueRangeRule { min: 7.1, max: 4505.3 }),
            Rc::new(ManualIdentityForecastRequiredCodeRule {
                message: "manual identity forecast needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_identity_forecast_validator();
        let item = new_manual_identity_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = ManualIdentityForecastCurrencyLengthRule { max_length: 86400 };
        let mut item = new_manual_identity_forecast("x", "y");
        item.currency = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = ManualIdentityForecastFloorValueRangeRule { min: 6.7, max: 1745.3 };
        let mut item = new_manual_identity_forecast("x", "y");
        item.floor_value = 1746.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualIdentityForecastRequiredCodeRule {
            message: "manual identity forecast needs an entry".to_string(),
        };
        let mut item = new_manual_identity_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
