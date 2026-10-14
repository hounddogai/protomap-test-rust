use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollout rule must pass. Check returns its problems, or none.
pub trait RolloutRuleRule: std::fmt::Debug {
    fn check(&self, item: &RolloutRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRuleAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl RolloutRuleRule for RolloutRuleAllowedSummaryRule {
    fn check(&self, item: &RolloutRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is inconsistent", item.summary)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRuleTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl RolloutRuleRule for RolloutRuleTagsLimitRule {
    fn check(&self, item: &RolloutRule) -> Vec<String> {
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
pub struct RolloutRuleSourceLengthRule {
    pub max_length: i64,
}

impl RolloutRuleRule for RolloutRuleSourceLengthRule {
    fn check(&self, item: &RolloutRule) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRuleCodeLengthRule {
    pub max_length: i64,
}

impl RolloutRuleRule for RolloutRuleCodeLengthRule {
    fn check(&self, item: &RolloutRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRuleAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutRuleRule for RolloutRuleAttemptsRangeRule {
    fn check(&self, item: &RolloutRule) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout rule.
#[derive(Debug, Clone)]
pub struct RolloutRuleValidator {
    pub rules: Vec<Rc<dyn RolloutRuleRule>>,
}

impl RolloutRuleValidator {
    pub fn validate(&self, item: &RolloutRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_rule_validator() -> RolloutRuleValidator {
    RolloutRuleValidator {
        rules: vec![
            Rc::new(RolloutRuleAllowedSummaryRule {
                allowed: vec!["strict".to_string(), "global".to_string()],
            }) as Rc<dyn RolloutRuleRule>,
            Rc::new(RolloutRuleTagsLimitRule {
                max_count: 120,
                banned: vec!["adaptive".to_string(), "seasonal".to_string()],
            }),
            Rc::new(RolloutRuleSourceLengthRule { max_length: 64 }),
            Rc::new(RolloutRuleCodeLengthRule { max_length: 80 }),
            Rc::new(RolloutRuleAttemptsRangeRule { min: 32, max: 3600 }),
        ],
    }
}
