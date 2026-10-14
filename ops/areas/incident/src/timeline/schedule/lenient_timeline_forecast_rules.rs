use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient timeline forecast must pass. Check returns its problems, or none.
pub trait LenientTimelineForecastRule: std::fmt::Debug {
    fn check(&self, item: &LenientTimelineForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelineForecastCodePrefixRule {
    pub prefix: String,
}

impl LenientTimelineForecastRule for LenientTimelineForecastCodePrefixRule {
    fn check(&self, item: &LenientTimelineForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelineForecastDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientTimelineForecastRule for LenientTimelineForecastDurationMinutesRangeRule {
    fn check(&self, item: &LenientTimelineForecast) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelineForecastRequiredCodeRule {
    pub message: String,
}

impl LenientTimelineForecastRule for LenientTimelineForecastRequiredCodeRule {
    fn check(&self, item: &LenientTimelineForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient timeline forecast.
#[derive(Debug, Clone)]
pub struct LenientTimelineForecastValidator {
    pub rules: Vec<Rc<dyn LenientTimelineForecastRule>>,
}

impl LenientTimelineForecastValidator {
    pub fn validate(&self, item: &LenientTimelineForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientTimelineForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientTimelineForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_timeline_forecast_validator() -> LenientTimelineForecastValidator {
    LenientTimelineForecastValidator {
        rules: vec![
            Rc::new(LenientTimelineForecastCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn LenientTimelineForecastRule>,
            Rc::new(LenientTimelineForecastDurationMinutesRangeRule { min: 120, max: 1000 }),
            Rc::new(LenientTimelineForecastRequiredCodeRule {
                message: "lenient timeline forecast needs this field".to_string(),
            }),
        ],
    }
}
