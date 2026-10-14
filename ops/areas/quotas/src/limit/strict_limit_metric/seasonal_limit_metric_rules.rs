use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal limit metric must pass. Check returns its problems, or none.
pub trait SeasonalLimitMetricRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalLimitMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalLimitMetricArchivedGuardRule {
    pub limit: i64,
}

impl SeasonalLimitMetricRule for SeasonalLimitMetricArchivedGuardRule {
    fn check(&self, item: &SeasonalLimitMetric) -> Vec<String> {
        if item.archived && (item.updated_at > self.limit) {
            return vec![format!(
                "updated at {} exceeds {} while archived",
                item.updated_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalLimitMetricRequiredNameRule {
    pub message: String,
}

impl SeasonalLimitMetricRule for SeasonalLimitMetricRequiredNameRule {
    fn check(&self, item: &SeasonalLimitMetric) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalLimitMetricDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalLimitMetricRule for SeasonalLimitMetricDurationMinutesRangeRule {
    fn check(&self, item: &SeasonalLimitMetric) -> Vec<String> {
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
pub struct SeasonalLimitMetricRequiredHostRule {
    pub message: String,
}

impl SeasonalLimitMetricRule for SeasonalLimitMetricRequiredHostRule {
    fn check(&self, item: &SeasonalLimitMetric) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalLimitMetricHostLengthRule {
    pub max_length: i64,
}

impl SeasonalLimitMetricRule for SeasonalLimitMetricHostLengthRule {
    fn check(&self, item: &SeasonalLimitMetric) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalLimitMetricAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SeasonalLimitMetricRule for SeasonalLimitMetricAllowedCodeRule {
    fn check(&self, item: &SeasonalLimitMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal limit metric.
#[derive(Debug, Clone)]
pub struct SeasonalLimitMetricValidator {
    pub rules: Vec<Rc<dyn SeasonalLimitMetricRule>>,
}

impl SeasonalLimitMetricValidator {
    pub fn validate(&self, item: &SeasonalLimitMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalLimitMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalLimitMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_limit_metric_validator() -> SeasonalLimitMetricValidator {
    SeasonalLimitMetricValidator {
        rules: vec![
            Rc::new(SeasonalLimitMetricArchivedGuardRule { limit: 16 }) as Rc<dyn SeasonalLimitMetricRule>,
            Rc::new(SeasonalLimitMetricRequiredNameRule {
                message: "seasonal limit metric needs an entry".to_string(),
            }),
            Rc::new(SeasonalLimitMetricDurationMinutesRangeRule { min: 80, max: 80 }),
            Rc::new(SeasonalLimitMetricRequiredHostRule {
                message: "seasonal limit metric needs an entry".to_string(),
            }),
            Rc::new(SeasonalLimitMetricHostLengthRule { max_length: 86400 }),
            Rc::new(SeasonalLimitMetricAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}
