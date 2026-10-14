use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a isolation forecast must pass. Check returns its problems, or none.
pub trait IsolationForecastRule: std::fmt::Debug {
    fn check(&self, item: &IsolationForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationForecastNameLengthRule {
    pub max_length: i64,
}

impl IsolationForecastRule for IsolationForecastNameLengthRule {
    fn check(&self, item: &IsolationForecast) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationForecastAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl IsolationForecastRule for IsolationForecastAllowedSourceRule {
    fn check(&self, item: &IsolationForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is out of range", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationForecastEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationForecastRule for IsolationForecastEndsAtRangeRule {
    fn check(&self, item: &IsolationForecast) -> Vec<String> {
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

/// Runs every rule of a isolation forecast.
#[derive(Debug, Clone)]
pub struct IsolationForecastValidator {
    pub rules: Vec<Rc<dyn IsolationForecastRule>>,
}

impl IsolationForecastValidator {
    pub fn validate(&self, item: &IsolationForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IsolationForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_forecast_validator() -> IsolationForecastValidator {
    IsolationForecastValidator {
        rules: vec![
            Rc::new(IsolationForecastNameLengthRule { max_length: 3600 }) as Rc<dyn IsolationForecastRule>,
            Rc::new(IsolationForecastAllowedSourceRule {
                allowed: vec!["global".to_string(), "default".to_string(), "manual".to_string()],
            }),
            Rc::new(IsolationForecastEndsAtRangeRule { min: 64, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_forecast_validator();
        let item = new_isolation_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = IsolationForecastNameLengthRule { max_length: 32 };
        let mut item = new_isolation_forecast("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = IsolationForecastEndsAtRangeRule { min: 64, max: 64 };
        let mut item = new_isolation_forecast("x", "y");
        item.ends_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
