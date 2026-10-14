use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive region pin override must pass. Check returns its problems, or none.
pub trait AdaptiveRegionPinOverrideRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRegionPinOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinOverrideScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveRegionPinOverrideRule for AdaptiveRegionPinOverrideScoreRangeRule {
    fn check(&self, item: &AdaptiveRegionPinOverride) -> Vec<String> {
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
pub struct AdaptiveRegionPinOverrideAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AdaptiveRegionPinOverrideRule for AdaptiveRegionPinOverrideAllowedCodeRule {
    fn check(&self, item: &AdaptiveRegionPinOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinOverrideTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl AdaptiveRegionPinOverrideRule for AdaptiveRegionPinOverrideTagsLimitRule {
    fn check(&self, item: &AdaptiveRegionPinOverride) -> Vec<String> {
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
pub struct AdaptiveRegionPinOverrideRequiredCodeRule {
    pub message: String,
}

impl AdaptiveRegionPinOverrideRule for AdaptiveRegionPinOverrideRequiredCodeRule {
    fn check(&self, item: &AdaptiveRegionPinOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive region pin override.
#[derive(Debug, Clone)]
pub struct AdaptiveRegionPinOverrideValidator {
    pub rules: Vec<Rc<dyn AdaptiveRegionPinOverrideRule>>,
}

impl AdaptiveRegionPinOverrideValidator {
    pub fn validate(&self, item: &AdaptiveRegionPinOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveRegionPinOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRegionPinOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_region_pin_override_validator() -> AdaptiveRegionPinOverrideValidator {
    AdaptiveRegionPinOverrideValidator {
        rules: vec![
            Rc::new(AdaptiveRegionPinOverrideScoreRangeRule { min: 4.1, max: 1599.0 })
                as Rc<dyn AdaptiveRegionPinOverrideRule>,
            Rc::new(AdaptiveRegionPinOverrideAllowedCodeRule {
                allowed: vec!["rolling".to_string()],
            }),
            Rc::new(AdaptiveRegionPinOverrideTagsLimitRule {
                max_count: 120,
                banned: vec!["regional".to_string(), "pending".to_string(), "global".to_string()],
            }),
            Rc::new(AdaptiveRegionPinOverrideRequiredCodeRule {
                message: "adaptive region pin override needs a value".to_string(),
            }),
        ],
    }
}
