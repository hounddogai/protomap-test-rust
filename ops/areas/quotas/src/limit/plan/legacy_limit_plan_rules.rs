use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy limit plan must pass. Check returns its problems, or none.
pub trait LegacyLimitPlanRule: std::fmt::Debug {
    fn check(&self, item: &LegacyLimitPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitPlanCodePrefixRule {
    pub prefix: String,
}

impl LegacyLimitPlanRule for LegacyLimitPlanCodePrefixRule {
    fn check(&self, item: &LegacyLimitPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitPlanAllowedNameRule {
    pub allowed: Vec<String>,
}

impl LegacyLimitPlanRule for LegacyLimitPlanAllowedNameRule {
    fn check(&self, item: &LegacyLimitPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is too long", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitPlanLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyLimitPlanRule for LegacyLimitPlanLimitRangeRule {
    fn check(&self, item: &LegacyLimitPlan) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitPlanWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyLimitPlanRule for LegacyLimitPlanWeightRangeRule {
    fn check(&self, item: &LegacyLimitPlan) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitPlanRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyLimitPlanRule for LegacyLimitPlanRetriesRangeRule {
    fn check(&self, item: &LegacyLimitPlan) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitPlanRequiredCodeRule {
    pub message: String,
}

impl LegacyLimitPlanRule for LegacyLimitPlanRequiredCodeRule {
    fn check(&self, item: &LegacyLimitPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy limit plan.
#[derive(Debug, Clone)]
pub struct LegacyLimitPlanValidator {
    pub rules: Vec<Rc<dyn LegacyLimitPlanRule>>,
}

impl LegacyLimitPlanValidator {
    pub fn validate(&self, item: &LegacyLimitPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyLimitPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyLimitPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_limit_plan_validator() -> LegacyLimitPlanValidator {
    LegacyLimitPlanValidator {
        rules: vec![
            Rc::new(LegacyLimitPlanCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn LegacyLimitPlanRule>,
            Rc::new(LegacyLimitPlanAllowedNameRule { allowed: Vec::new() }),
            Rc::new(LegacyLimitPlanLimitRangeRule { min: 80, max: 1000 }),
            Rc::new(LegacyLimitPlanWeightRangeRule { min: 3.4, max: 2638.0 }),
            Rc::new(LegacyLimitPlanRetriesRangeRule { min: 64, max: 1000 }),
            Rc::new(LegacyLimitPlanRequiredCodeRule {
                message: "legacy limit plan needs a value".to_string(),
            }),
        ],
    }
}
