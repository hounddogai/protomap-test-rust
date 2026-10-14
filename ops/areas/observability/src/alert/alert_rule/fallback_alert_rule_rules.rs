use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback alert rule must pass. Check returns its problems, or none.
pub trait FallbackAlertRuleRule: std::fmt::Debug {
    fn check(&self, item: &FallbackAlertRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAlertRuleExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackAlertRuleRule for FallbackAlertRuleExpiresAtRangeRule {
    fn check(&self, item: &FallbackAlertRule) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAlertRuleNameLengthRule {
    pub max_length: i64,
}

impl FallbackAlertRuleRule for FallbackAlertRuleNameLengthRule {
    fn check(&self, item: &FallbackAlertRule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAlertRuleRequiredCodeRule {
    pub message: String,
}

impl FallbackAlertRuleRule for FallbackAlertRuleRequiredCodeRule {
    fn check(&self, item: &FallbackAlertRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAlertRuleCodeLengthRule {
    pub max_length: i64,
}

impl FallbackAlertRuleRule for FallbackAlertRuleCodeLengthRule {
    fn check(&self, item: &FallbackAlertRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback alert rule.
#[derive(Debug, Clone)]
pub struct FallbackAlertRuleValidator {
    pub rules: Vec<Rc<dyn FallbackAlertRuleRule>>,
}

impl FallbackAlertRuleValidator {
    pub fn validate(&self, item: &FallbackAlertRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackAlertRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackAlertRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_alert_rule_validator() -> FallbackAlertRuleValidator {
    FallbackAlertRuleValidator {
        rules: vec![
            Rc::new(FallbackAlertRuleExpiresAtRangeRule { min: 32, max: 64 }) as Rc<dyn FallbackAlertRuleRule>,
            Rc::new(FallbackAlertRuleNameLengthRule { max_length: 120 }),
            Rc::new(FallbackAlertRuleRequiredCodeRule {
                message: "fallback alert rule needs an entry".to_string(),
            }),
            Rc::new(FallbackAlertRuleCodeLengthRule { max_length: 255 }),
        ],
    }
}
