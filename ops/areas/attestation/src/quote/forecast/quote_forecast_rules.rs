use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote forecast must pass. Check returns its problems, or none.
pub trait QuoteForecastRule: std::fmt::Debug {
    fn check(&self, item: &QuoteForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteForecastRequiredSourceRule {
    pub message: String,
}

impl QuoteForecastRule for QuoteForecastRequiredSourceRule {
    fn check(&self, item: &QuoteForecast) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteForecastSourceLengthRule {
    pub max_length: i64,
}

impl QuoteForecastRule for QuoteForecastSourceLengthRule {
    fn check(&self, item: &QuoteForecast) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteForecastReviewerLengthRule {
    pub max_length: i64,
}

impl QuoteForecastRule for QuoteForecastReviewerLengthRule {
    fn check(&self, item: &QuoteForecast) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteForecastScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QuoteForecastRule for QuoteForecastScoreRangeRule {
    fn check(&self, item: &QuoteForecast) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteForecastCodeLengthRule {
    pub max_length: i64,
}

impl QuoteForecastRule for QuoteForecastCodeLengthRule {
    fn check(&self, item: &QuoteForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote forecast.
#[derive(Debug, Clone)]
pub struct QuoteForecastValidator {
    pub rules: Vec<Rc<dyn QuoteForecastRule>>,
}

impl QuoteForecastValidator {
    pub fn validate(&self, item: &QuoteForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_forecast_validator() -> QuoteForecastValidator {
    QuoteForecastValidator {
        rules: vec![
            Rc::new(QuoteForecastRequiredSourceRule {
                message: "quote forecast needs an entry".to_string(),
            }) as Rc<dyn QuoteForecastRule>,
            Rc::new(QuoteForecastSourceLengthRule { max_length: 16 }),
            Rc::new(QuoteForecastReviewerLengthRule { max_length: 3600 }),
            Rc::new(QuoteForecastScoreRangeRule { min: 1.3, max: 3145.4 }),
            Rc::new(QuoteForecastCodeLengthRule { max_length: 1000 }),
        ],
    }
}
