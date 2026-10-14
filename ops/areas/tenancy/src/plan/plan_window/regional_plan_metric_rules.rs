use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional plan metric must pass. Check returns its problems, or none.
pub trait RegionalPlanMetricRule: std::fmt::Debug {
    fn check(&self, item: &RegionalPlanMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPlanMetricCodePrefixRule {
    pub prefix: String,
}

impl RegionalPlanMetricRule for RegionalPlanMetricCodePrefixRule {
    fn check(&self, item: &RegionalPlanMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPlanMetricCodeLengthRule {
    pub max_length: i64,
}

impl RegionalPlanMetricRule for RegionalPlanMetricCodeLengthRule {
    fn check(&self, item: &RegionalPlanMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPlanMetricAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RegionalPlanMetricRule for RegionalPlanMetricAllowedNameRule {
    fn check(&self, item: &RegionalPlanMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPlanMetricCountryLengthRule {
    pub max_length: i64,
}

impl RegionalPlanMetricRule for RegionalPlanMetricCountryLengthRule {
    fn check(&self, item: &RegionalPlanMetric) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPlanMetricCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalPlanMetricRule for RegionalPlanMetricCeilingRangeRule {
    fn check(&self, item: &RegionalPlanMetric) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPlanMetricNameLengthRule {
    pub max_length: i64,
}

impl RegionalPlanMetricRule for RegionalPlanMetricNameLengthRule {
    fn check(&self, item: &RegionalPlanMetric) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional plan metric.
#[derive(Debug, Clone)]
pub struct RegionalPlanMetricValidator {
    pub rules: Vec<Rc<dyn RegionalPlanMetricRule>>,
}

impl RegionalPlanMetricValidator {
    pub fn validate(&self, item: &RegionalPlanMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalPlanMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalPlanMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_plan_metric_validator() -> RegionalPlanMetricValidator {
    RegionalPlanMetricValidator {
        rules: vec![
            Rc::new(RegionalPlanMetricCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn RegionalPlanMetricRule>,
            Rc::new(RegionalPlanMetricCodeLengthRule { max_length: 32 }),
            Rc::new(RegionalPlanMetricAllowedNameRule { allowed: Vec::new() }),
            Rc::new(RegionalPlanMetricCountryLengthRule { max_length: 255 }),
            Rc::new(RegionalPlanMetricCeilingRangeRule { min: 1.1, max: 2828.7 }),
            Rc::new(RegionalPlanMetricNameLengthRule { max_length: 255 }),
        ],
    }
}
