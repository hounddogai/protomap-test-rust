use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a timeline group must pass. Check returns its problems, or none.
pub trait TimelineGroupRule: std::fmt::Debug {
    fn check(&self, item: &TimelineGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineGroupRequiredIssuerRule {
    pub message: String,
}

impl TimelineGroupRule for TimelineGroupRequiredIssuerRule {
    fn check(&self, item: &TimelineGroup) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineGroupQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineGroupRule for TimelineGroupQuantityRangeRule {
    fn check(&self, item: &TimelineGroup) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineGroupExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineGroupRule for TimelineGroupExpiresAtRangeRule {
    fn check(&self, item: &TimelineGroup) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineGroupCodePrefixRule {
    pub prefix: String,
}

impl TimelineGroupRule for TimelineGroupCodePrefixRule {
    fn check(&self, item: &TimelineGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineGroupRequiredCodeRule {
    pub message: String,
}

impl TimelineGroupRule for TimelineGroupRequiredCodeRule {
    fn check(&self, item: &TimelineGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineGroupAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl TimelineGroupRule for TimelineGroupAllowedIssuerRule {
    fn check(&self, item: &TimelineGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is blocked", item.issuer)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline group.
#[derive(Debug, Clone)]
pub struct TimelineGroupValidator {
    pub rules: Vec<Rc<dyn TimelineGroupRule>>,
}

impl TimelineGroupValidator {
    pub fn validate(&self, item: &TimelineGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_group_validator() -> TimelineGroupValidator {
    TimelineGroupValidator {
        rules: vec![
            Rc::new(TimelineGroupRequiredIssuerRule {
                message: "timeline group needs this field".to_string(),
            }) as Rc<dyn TimelineGroupRule>,
            Rc::new(TimelineGroupQuantityRangeRule { min: 80, max: 1000 }),
            Rc::new(TimelineGroupExpiresAtRangeRule { min: 120, max: 3600 }),
            Rc::new(TimelineGroupCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(TimelineGroupRequiredCodeRule {
                message: "timeline group needs an entry".to_string(),
            }),
            Rc::new(TimelineGroupAllowedIssuerRule { allowed: Vec::new() }),
        ],
    }
}
