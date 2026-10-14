use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default suspension metric must pass. Check returns its problems, or none.
pub trait DefaultSuspensionMetricRule: std::fmt::Debug {
    fn check(&self, item: &DefaultSuspensionMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSuspensionMetricCodeLengthRule {
    pub max_length: i64,
}

impl DefaultSuspensionMetricRule for DefaultSuspensionMetricCodeLengthRule {
    fn check(&self, item: &DefaultSuspensionMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSuspensionMetricTitleLengthRule {
    pub max_length: i64,
}

impl DefaultSuspensionMetricRule for DefaultSuspensionMetricTitleLengthRule {
    fn check(&self, item: &DefaultSuspensionMetric) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSuspensionMetricRequiredTimezoneRule {
    pub message: String,
}

impl DefaultSuspensionMetricRule for DefaultSuspensionMetricRequiredTimezoneRule {
    fn check(&self, item: &DefaultSuspensionMetric) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSuspensionMetricAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl DefaultSuspensionMetricRule for DefaultSuspensionMetricAllowedTitleRule {
    fn check(&self, item: &DefaultSuspensionMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is duplicated", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSuspensionMetricArchivedGuardRule {
    pub limit: i64,
}

impl DefaultSuspensionMetricRule for DefaultSuspensionMetricArchivedGuardRule {
    fn check(&self, item: &DefaultSuspensionMetric) -> Vec<String> {
        if item.archived && (item.duration_minutes > self.limit) {
            return vec![format!(
                "duration minutes {} exceeds {} while archived",
                item.duration_minutes, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a default suspension metric.
#[derive(Debug, Clone)]
pub struct DefaultSuspensionMetricValidator {
    pub rules: Vec<Rc<dyn DefaultSuspensionMetricRule>>,
}

impl DefaultSuspensionMetricValidator {
    pub fn validate(&self, item: &DefaultSuspensionMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultSuspensionMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultSuspensionMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_suspension_metric_validator() -> DefaultSuspensionMetricValidator {
    DefaultSuspensionMetricValidator {
        rules: vec![
            Rc::new(DefaultSuspensionMetricCodeLengthRule { max_length: 120 }) as Rc<dyn DefaultSuspensionMetricRule>,
            Rc::new(DefaultSuspensionMetricTitleLengthRule { max_length: 86400 }),
            Rc::new(DefaultSuspensionMetricRequiredTimezoneRule {
                message: "default suspension metric needs an entry".to_string(),
            }),
            Rc::new(DefaultSuspensionMetricAllowedTitleRule {
                allowed: vec!["scheduled".to_string()],
            }),
            Rc::new(DefaultSuspensionMetricArchivedGuardRule { limit: 80 }),
        ],
    }
}
