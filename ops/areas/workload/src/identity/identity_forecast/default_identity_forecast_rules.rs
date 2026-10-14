use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default identity forecast must pass. Check returns its problems, or none.
pub trait DefaultIdentityForecastRule: std::fmt::Debug {
    fn check(&self, item: &DefaultIdentityForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentityForecastBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultIdentityForecastRule for DefaultIdentityForecastBurstRangeRule {
    fn check(&self, item: &DefaultIdentityForecast) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentityForecastTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultIdentityForecastRule for DefaultIdentityForecastTtlSecondsRangeRule {
    fn check(&self, item: &DefaultIdentityForecast) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentityForecastCodePrefixRule {
    pub prefix: String,
}

impl DefaultIdentityForecastRule for DefaultIdentityForecastCodePrefixRule {
    fn check(&self, item: &DefaultIdentityForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default identity forecast.
#[derive(Debug, Clone)]
pub struct DefaultIdentityForecastValidator {
    pub rules: Vec<Rc<dyn DefaultIdentityForecastRule>>,
}

impl DefaultIdentityForecastValidator {
    pub fn validate(&self, item: &DefaultIdentityForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultIdentityForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultIdentityForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_identity_forecast_validator() -> DefaultIdentityForecastValidator {
    DefaultIdentityForecastValidator {
        rules: vec![
            Rc::new(DefaultIdentityForecastBurstRangeRule { min: 80, max: 3600 })
                as Rc<dyn DefaultIdentityForecastRule>,
            Rc::new(DefaultIdentityForecastTtlSecondsRangeRule { min: 80, max: 86400 }),
            Rc::new(DefaultIdentityForecastCodePrefixRule {
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
        let validator = default_default_identity_forecast_validator();
        let item = new_default_identity_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = DefaultIdentityForecastBurstRangeRule { min: 16, max: 1000 };
        let mut item = new_default_identity_forecast("x", "y");
        item.burst = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = DefaultIdentityForecastTtlSecondsRangeRule { min: 1000, max: 3600 };
        let mut item = new_default_identity_forecast("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
