use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled egress profile must pass. Check returns its problems, or none.
pub trait ScheduledEgressProfileRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledEgressProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressProfileCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledEgressProfileRule for ScheduledEgressProfileCodeLengthRule {
    fn check(&self, item: &ScheduledEgressProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressProfileAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ScheduledEgressProfileRule for ScheduledEgressProfileAllowedTitleRule {
    fn check(&self, item: &ScheduledEgressProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is expired", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressProfileMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledEgressProfileRule for ScheduledEgressProfileMarginRangeRule {
    fn check(&self, item: &ScheduledEgressProfile) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressProfileChannelLengthRule {
    pub max_length: i64,
}

impl ScheduledEgressProfileRule for ScheduledEgressProfileChannelLengthRule {
    fn check(&self, item: &ScheduledEgressProfile) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressProfileRequiredCountryRule {
    pub message: String,
}

impl ScheduledEgressProfileRule for ScheduledEgressProfileRequiredCountryRule {
    fn check(&self, item: &ScheduledEgressProfile) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressProfileWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledEgressProfileRule for ScheduledEgressProfileWeightRangeRule {
    fn check(&self, item: &ScheduledEgressProfile) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a scheduled egress profile.
#[derive(Debug, Clone)]
pub struct ScheduledEgressProfileValidator {
    pub rules: Vec<Rc<dyn ScheduledEgressProfileRule>>,
}

impl ScheduledEgressProfileValidator {
    pub fn validate(&self, item: &ScheduledEgressProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledEgressProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledEgressProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_egress_profile_validator() -> ScheduledEgressProfileValidator {
    ScheduledEgressProfileValidator {
        rules: vec![
            Rc::new(ScheduledEgressProfileCodeLengthRule { max_length: 120 }) as Rc<dyn ScheduledEgressProfileRule>,
            Rc::new(ScheduledEgressProfileAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(ScheduledEgressProfileMarginRangeRule { min: 1.5, max: 1099.8 }),
            Rc::new(ScheduledEgressProfileChannelLengthRule { max_length: 3600 }),
            Rc::new(ScheduledEgressProfileRequiredCountryRule {
                message: "scheduled egress profile needs an entry".to_string(),
            }),
            Rc::new(ScheduledEgressProfileWeightRangeRule { min: 6.9, max: 181.7 }),
        ],
    }
}
