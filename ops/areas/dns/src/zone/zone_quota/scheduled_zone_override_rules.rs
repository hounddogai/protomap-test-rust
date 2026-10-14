use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled zone override must pass. Check returns its problems, or none.
pub trait ScheduledZoneOverrideRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledZoneOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneOverrideAllowedNameRule {
    pub allowed: Vec<String>,
}

impl ScheduledZoneOverrideRule for ScheduledZoneOverrideAllowedNameRule {
    fn check(&self, item: &ScheduledZoneOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneOverrideSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledZoneOverrideRule for ScheduledZoneOverrideSequenceRangeRule {
    fn check(&self, item: &ScheduledZoneOverride) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneOverrideCodePrefixRule {
    pub prefix: String,
}

impl ScheduledZoneOverrideRule for ScheduledZoneOverrideCodePrefixRule {
    fn check(&self, item: &ScheduledZoneOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneOverrideRequiredHostRule {
    pub message: String,
}

impl ScheduledZoneOverrideRule for ScheduledZoneOverrideRequiredHostRule {
    fn check(&self, item: &ScheduledZoneOverride) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneOverrideRequiredNameRule {
    pub message: String,
}

impl ScheduledZoneOverrideRule for ScheduledZoneOverrideRequiredNameRule {
    fn check(&self, item: &ScheduledZoneOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneOverrideRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledZoneOverrideRule for ScheduledZoneOverrideRotationDaysRangeRule {
    fn check(&self, item: &ScheduledZoneOverride) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled zone override.
#[derive(Debug, Clone)]
pub struct ScheduledZoneOverrideValidator {
    pub rules: Vec<Rc<dyn ScheduledZoneOverrideRule>>,
}

impl ScheduledZoneOverrideValidator {
    pub fn validate(&self, item: &ScheduledZoneOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledZoneOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledZoneOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_zone_override_validator() -> ScheduledZoneOverrideValidator {
    ScheduledZoneOverrideValidator {
        rules: vec![
            Rc::new(ScheduledZoneOverrideAllowedNameRule {
                allowed: vec!["default".to_string(), "global".to_string(), "manual".to_string()],
            }) as Rc<dyn ScheduledZoneOverrideRule>,
            Rc::new(ScheduledZoneOverrideSequenceRangeRule { min: 32, max: 3600 }),
            Rc::new(ScheduledZoneOverrideCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ScheduledZoneOverrideRequiredHostRule {
                message: "scheduled zone override needs a value".to_string(),
            }),
            Rc::new(ScheduledZoneOverrideRequiredNameRule {
                message: "scheduled zone override needs a value".to_string(),
            }),
            Rc::new(ScheduledZoneOverrideRotationDaysRangeRule { min: 16, max: 64 }),
        ],
    }
}
