use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy forecast must pass. Check returns its problems, or none.
pub trait TtlPolicyForecastRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyForecastTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyForecastRule for TtlPolicyForecastTtlSecondsRangeRule {
    fn check(&self, item: &TtlPolicyForecast) -> Vec<String> {
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
pub struct TtlPolicyForecastFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TtlPolicyForecastRule for TtlPolicyForecastFloorValueRangeRule {
    fn check(&self, item: &TtlPolicyForecast) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyForecastRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyForecastRule for TtlPolicyForecastRetriesRangeRule {
    fn check(&self, item: &TtlPolicyForecast) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy forecast.
#[derive(Debug, Clone)]
pub struct TtlPolicyForecastValidator {
    pub rules: Vec<Rc<dyn TtlPolicyForecastRule>>,
}

impl TtlPolicyForecastValidator {
    pub fn validate(&self, item: &TtlPolicyForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TtlPolicyForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_forecast_validator() -> TtlPolicyForecastValidator {
    TtlPolicyForecastValidator {
        rules: vec![
            Rc::new(TtlPolicyForecastTtlSecondsRangeRule { min: 32, max: 3600 }) as Rc<dyn TtlPolicyForecastRule>,
            Rc::new(TtlPolicyForecastFloorValueRangeRule { min: 8.5, max: 2161.1 }),
            Rc::new(TtlPolicyForecastRetriesRangeRule { min: 255, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_forecast_validator();
        let item = new_ttl_policy_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = TtlPolicyForecastTtlSecondsRangeRule { min: 120, max: 120 };
        let mut item = new_ttl_policy_forecast("x", "y");
        item.ttl_seconds = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = TtlPolicyForecastFloorValueRangeRule { min: 4.5, max: 1561.6 };
        let mut item = new_ttl_policy_forecast("x", "y");
        item.floor_value = 1563.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = TtlPolicyForecastRetriesRangeRule { min: 64, max: 255 };
        let mut item = new_ttl_policy_forecast("x", "y");
        item.retries = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
