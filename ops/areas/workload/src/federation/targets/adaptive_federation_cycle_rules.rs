use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive federation cycle must pass. Check returns its problems, or none.
pub trait AdaptiveFederationCycleRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveFederationCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFederationCycleUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveFederationCycleRule for AdaptiveFederationCycleUpdatedAtRangeRule {
    fn check(&self, item: &AdaptiveFederationCycle) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFederationCycleChannelLengthRule {
    pub max_length: i64,
}

impl AdaptiveFederationCycleRule for AdaptiveFederationCycleChannelLengthRule {
    fn check(&self, item: &AdaptiveFederationCycle) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFederationCycleCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveFederationCycleRule for AdaptiveFederationCycleCodePrefixRule {
    fn check(&self, item: &AdaptiveFederationCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFederationCycleLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveFederationCycleRule for AdaptiveFederationCycleLimitRangeRule {
    fn check(&self, item: &AdaptiveFederationCycle) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive federation cycle.
#[derive(Debug, Clone)]
pub struct AdaptiveFederationCycleValidator {
    pub rules: Vec<Rc<dyn AdaptiveFederationCycleRule>>,
}

impl AdaptiveFederationCycleValidator {
    pub fn validate(&self, item: &AdaptiveFederationCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveFederationCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveFederationCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_federation_cycle_validator() -> AdaptiveFederationCycleValidator {
    AdaptiveFederationCycleValidator {
        rules: vec![
            Rc::new(AdaptiveFederationCycleUpdatedAtRangeRule { min: 64, max: 255 })
                as Rc<dyn AdaptiveFederationCycleRule>,
            Rc::new(AdaptiveFederationCycleChannelLengthRule { max_length: 255 }),
            Rc::new(AdaptiveFederationCycleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(AdaptiveFederationCycleLimitRangeRule { min: 64, max: 80 }),
        ],
    }
}
