use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a postmortem baseline must pass. Check returns its problems, or none.
pub trait PostmortemBaselineRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBaselineThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PostmortemBaselineRule for PostmortemBaselineThresholdRangeRule {
    fn check(&self, item: &PostmortemBaseline) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBaselineRequiredCodeRule {
    pub message: String,
}

impl PostmortemBaselineRule for PostmortemBaselineRequiredCodeRule {
    fn check(&self, item: &PostmortemBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBaselineScopeLengthRule {
    pub max_length: i64,
}

impl PostmortemBaselineRule for PostmortemBaselineScopeLengthRule {
    fn check(&self, item: &PostmortemBaseline) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBaselineAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PostmortemBaselineRule for PostmortemBaselineAllowedLabelRule {
    fn check(&self, item: &PostmortemBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is stale", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBaselineBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemBaselineRule for PostmortemBaselineBatchSizeRangeRule {
    fn check(&self, item: &PostmortemBaseline) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a postmortem baseline.
#[derive(Debug, Clone)]
pub struct PostmortemBaselineValidator {
    pub rules: Vec<Rc<dyn PostmortemBaselineRule>>,
}

impl PostmortemBaselineValidator {
    pub fn validate(&self, item: &PostmortemBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_baseline_validator() -> PostmortemBaselineValidator {
    PostmortemBaselineValidator {
        rules: vec![
            Rc::new(PostmortemBaselineThresholdRangeRule { min: 6.3, max: 810.0 }) as Rc<dyn PostmortemBaselineRule>,
            Rc::new(PostmortemBaselineRequiredCodeRule {
                message: "postmortem baseline needs this field".to_string(),
            }),
            Rc::new(PostmortemBaselineScopeLengthRule { max_length: 255 }),
            Rc::new(PostmortemBaselineAllowedLabelRule {
                allowed: vec!["strict".to_string(), "global".to_string(), "pending".to_string()],
            }),
            Rc::new(PostmortemBaselineBatchSizeRangeRule { min: 120, max: 120 }),
        ],
    }
}
