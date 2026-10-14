use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page metric must pass. Check returns its problems, or none.
pub trait PageMetricRule: std::fmt::Debug {
    fn check(&self, item: &PageMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageMetricRequiredPhoneRule {
    pub message: String,
}

impl PageMetricRule for PageMetricRequiredPhoneRule {
    fn check(&self, item: &PageMetric) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageMetricEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageMetricRule for PageMetricEndsAtRangeRule {
    fn check(&self, item: &PageMetric) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageMetricCodePrefixRule {
    pub prefix: String,
}

impl PageMetricRule for PageMetricCodePrefixRule {
    fn check(&self, item: &PageMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageMetricTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl PageMetricRule for PageMetricTagsLimitRule {
    fn check(&self, item: &PageMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageMetricCodeLengthRule {
    pub max_length: i64,
}

impl PageMetricRule for PageMetricCodeLengthRule {
    fn check(&self, item: &PageMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a page metric.
#[derive(Debug, Clone)]
pub struct PageMetricValidator {
    pub rules: Vec<Rc<dyn PageMetricRule>>,
}

impl PageMetricValidator {
    pub fn validate(&self, item: &PageMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PageMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_metric_validator() -> PageMetricValidator {
    PageMetricValidator {
        rules: vec![
            Rc::new(PageMetricRequiredPhoneRule {
                message: "page metric needs a value".to_string(),
            }) as Rc<dyn PageMetricRule>,
            Rc::new(PageMetricEndsAtRangeRule { min: 16, max: 64 }),
            Rc::new(PageMetricCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(PageMetricTagsLimitRule {
                max_count: 120,
                banned: Vec::new(),
            }),
            Rc::new(PageMetricCodeLengthRule { max_length: 80 }),
        ],
    }
}
