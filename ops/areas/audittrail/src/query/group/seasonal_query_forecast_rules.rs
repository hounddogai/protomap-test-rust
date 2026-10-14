use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal query forecast must pass. Check returns its problems, or none.
pub trait SeasonalQueryForecastRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalQueryForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryForecastRequiredNameRule {
    pub message: String,
}

impl SeasonalQueryForecastRule for SeasonalQueryForecastRequiredNameRule {
    fn check(&self, item: &SeasonalQueryForecast) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryForecastKeyIdLengthRule {
    pub max_length: i64,
}

impl SeasonalQueryForecastRule for SeasonalQueryForecastKeyIdLengthRule {
    fn check(&self, item: &SeasonalQueryForecast) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryForecastPhoneLengthRule {
    pub max_length: i64,
}

impl SeasonalQueryForecastRule for SeasonalQueryForecastPhoneLengthRule {
    fn check(&self, item: &SeasonalQueryForecast) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal query forecast.
#[derive(Debug, Clone)]
pub struct SeasonalQueryForecastValidator {
    pub rules: Vec<Rc<dyn SeasonalQueryForecastRule>>,
}

impl SeasonalQueryForecastValidator {
    pub fn validate(&self, item: &SeasonalQueryForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalQueryForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalQueryForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_query_forecast_validator() -> SeasonalQueryForecastValidator {
    SeasonalQueryForecastValidator {
        rules: vec![
            Rc::new(SeasonalQueryForecastRequiredNameRule {
                message: "seasonal query forecast needs a value".to_string(),
            }) as Rc<dyn SeasonalQueryForecastRule>,
            Rc::new(SeasonalQueryForecastKeyIdLengthRule { max_length: 1000 }),
            Rc::new(SeasonalQueryForecastPhoneLengthRule { max_length: 86400 }),
        ],
    }
}
