use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global forecast baseline must pass. Check returns its problems, or none.
pub trait GlobalForecastBaselineRule: std::fmt::Debug {
    fn check(&self, item: &GlobalForecastBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastBaselineRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalForecastBaselineRule for GlobalForecastBaselineRevisionRangeRule {
    fn check(&self, item: &GlobalForecastBaseline) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastBaselineCodePrefixRule {
    pub prefix: String,
}

impl GlobalForecastBaselineRule for GlobalForecastBaselineCodePrefixRule {
    fn check(&self, item: &GlobalForecastBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastBaselineTitleLengthRule {
    pub max_length: i64,
}

impl GlobalForecastBaselineRule for GlobalForecastBaselineTitleLengthRule {
    fn check(&self, item: &GlobalForecastBaseline) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastBaselinePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalForecastBaselineRule for GlobalForecastBaselinePortRangeRule {
    fn check(&self, item: &GlobalForecastBaseline) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global forecast baseline.
#[derive(Debug, Clone)]
pub struct GlobalForecastBaselineValidator {
    pub rules: Vec<Rc<dyn GlobalForecastBaselineRule>>,
}

impl GlobalForecastBaselineValidator {
    pub fn validate(&self, item: &GlobalForecastBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalForecastBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalForecastBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_forecast_baseline_validator() -> GlobalForecastBaselineValidator {
    GlobalForecastBaselineValidator {
        rules: vec![
            Rc::new(GlobalForecastBaselineRevisionRangeRule { min: 32, max: 64 }) as Rc<dyn GlobalForecastBaselineRule>,
            Rc::new(GlobalForecastBaselineCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(GlobalForecastBaselineTitleLengthRule { max_length: 3600 }),
            Rc::new(GlobalForecastBaselinePortRangeRule { min: 32, max: 120 }),
        ],
    }
}
