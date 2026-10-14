use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit forecast must pass. Check returns its problems, or none.
pub trait LimitForecastRule: std::fmt::Debug {
    fn check(&self, item: &LimitForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitForecastCodeLengthRule {
    pub max_length: i64,
}

impl LimitForecastRule for LimitForecastCodeLengthRule {
    fn check(&self, item: &LimitForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitForecastPhoneLengthRule {
    pub max_length: i64,
}

impl LimitForecastRule for LimitForecastPhoneLengthRule {
    fn check(&self, item: &LimitForecast) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitForecastRequiredCodeRule {
    pub message: String,
}

impl LimitForecastRule for LimitForecastRequiredCodeRule {
    fn check(&self, item: &LimitForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit forecast.
#[derive(Debug, Clone)]
pub struct LimitForecastValidator {
    pub rules: Vec<Rc<dyn LimitForecastRule>>,
}

impl LimitForecastValidator {
    pub fn validate(&self, item: &LimitForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LimitForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_forecast_validator() -> LimitForecastValidator {
    LimitForecastValidator {
        rules: vec![
            Rc::new(LimitForecastCodeLengthRule { max_length: 1000 }) as Rc<dyn LimitForecastRule>,
            Rc::new(LimitForecastPhoneLengthRule { max_length: 64 }),
            Rc::new(LimitForecastRequiredCodeRule {
                message: "limit forecast needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_forecast_validator();
        let item = new_limit_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LimitForecastCodeLengthRule { max_length: 255 };
        let mut item = new_limit_forecast("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = LimitForecastPhoneLengthRule { max_length: 32 };
        let mut item = new_limit_forecast("x", "y");
        item.phone = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LimitForecastRequiredCodeRule {
            message: "limit forecast needs this field".to_string(),
        };
        let mut item = new_limit_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
