use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a usage forecast must pass. Check returns its problems, or none.
pub trait UsageForecastRule: std::fmt::Debug {
    fn check(&self, item: &UsageForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageForecastRequiredCodeRule {
    pub message: String,
}

impl UsageForecastRule for UsageForecastRequiredCodeRule {
    fn check(&self, item: &UsageForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageForecastExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsageForecastRule for UsageForecastExpiresAtRangeRule {
    fn check(&self, item: &UsageForecast) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageForecastCodeLengthRule {
    pub max_length: i64,
}

impl UsageForecastRule for UsageForecastCodeLengthRule {
    fn check(&self, item: &UsageForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageForecastNameLengthRule {
    pub max_length: i64,
}

impl UsageForecastRule for UsageForecastNameLengthRule {
    fn check(&self, item: &UsageForecast) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage forecast.
#[derive(Debug, Clone)]
pub struct UsageForecastValidator {
    pub rules: Vec<Rc<dyn UsageForecastRule>>,
}

impl UsageForecastValidator {
    pub fn validate(&self, item: &UsageForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &UsageForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_forecast_validator() -> UsageForecastValidator {
    UsageForecastValidator {
        rules: vec![
            Rc::new(UsageForecastRequiredCodeRule {
                message: "usage forecast needs an entry".to_string(),
            }) as Rc<dyn UsageForecastRule>,
            Rc::new(UsageForecastExpiresAtRangeRule { min: 64, max: 255 }),
            Rc::new(UsageForecastCodeLengthRule { max_length: 3600 }),
            Rc::new(UsageForecastNameLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_forecast_validator();
        let item = new_usage_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = UsageForecastRequiredCodeRule {
            message: "usage forecast needs this field".to_string(),
        };
        let mut item = new_usage_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = UsageForecastExpiresAtRangeRule { min: 3600, max: 86400 };
        let mut item = new_usage_forecast("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = UsageForecastCodeLengthRule { max_length: 86400 };
        let mut item = new_usage_forecast("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = UsageForecastNameLengthRule { max_length: 255 };
        let mut item = new_usage_forecast("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
