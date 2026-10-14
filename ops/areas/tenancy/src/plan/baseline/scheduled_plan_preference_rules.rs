use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled plan preference must pass. Check returns its problems, or none.
pub trait ScheduledPlanPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledPlanPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPlanPreferenceTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledPlanPreferenceRule for ScheduledPlanPreferenceTtlSecondsRangeRule {
    fn check(&self, item: &ScheduledPlanPreference) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPlanPreferenceCodePrefixRule {
    pub prefix: String,
}

impl ScheduledPlanPreferenceRule for ScheduledPlanPreferenceCodePrefixRule {
    fn check(&self, item: &ScheduledPlanPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPlanPreferenceTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl ScheduledPlanPreferenceRule for ScheduledPlanPreferenceTagsLimitRule {
    fn check(&self, item: &ScheduledPlanPreference) -> Vec<String> {
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
pub struct ScheduledPlanPreferenceNameLengthRule {
    pub max_length: i64,
}

impl ScheduledPlanPreferenceRule for ScheduledPlanPreferenceNameLengthRule {
    fn check(&self, item: &ScheduledPlanPreference) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPlanPreferenceTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledPlanPreferenceRule for ScheduledPlanPreferenceTitleLengthRule {
    fn check(&self, item: &ScheduledPlanPreference) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPlanPreferenceRequiredCategoryRule {
    pub message: String,
}

impl ScheduledPlanPreferenceRule for ScheduledPlanPreferenceRequiredCategoryRule {
    fn check(&self, item: &ScheduledPlanPreference) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled plan preference.
#[derive(Debug, Clone)]
pub struct ScheduledPlanPreferenceValidator {
    pub rules: Vec<Rc<dyn ScheduledPlanPreferenceRule>>,
}

impl ScheduledPlanPreferenceValidator {
    pub fn validate(&self, item: &ScheduledPlanPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledPlanPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledPlanPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_plan_preference_validator() -> ScheduledPlanPreferenceValidator {
    ScheduledPlanPreferenceValidator {
        rules: vec![
            Rc::new(ScheduledPlanPreferenceTtlSecondsRangeRule { min: 255, max: 3600 })
                as Rc<dyn ScheduledPlanPreferenceRule>,
            Rc::new(ScheduledPlanPreferenceCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ScheduledPlanPreferenceTagsLimitRule {
                max_count: 86400,
                banned: vec!["legacy".to_string(), "regional".to_string()],
            }),
            Rc::new(ScheduledPlanPreferenceNameLengthRule { max_length: 80 }),
            Rc::new(ScheduledPlanPreferenceTitleLengthRule { max_length: 32 }),
            Rc::new(ScheduledPlanPreferenceRequiredCategoryRule {
                message: "scheduled plan preference needs a value".to_string(),
            }),
        ],
    }
}
