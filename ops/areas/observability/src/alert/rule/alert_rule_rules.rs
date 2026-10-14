use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a alert rule must pass. Check returns its problems, or none.
pub trait AlertRuleRule: std::fmt::Debug {
    fn check(&self, item: &AlertRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRuleKeyIdLengthRule {
    pub max_length: i64,
}

impl AlertRuleRule for AlertRuleKeyIdLengthRule {
    fn check(&self, item: &AlertRule) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRuleRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertRuleRule for AlertRuleRetriesRangeRule {
    fn check(&self, item: &AlertRule) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRuleUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertRuleRule for AlertRuleUpdatedAtRangeRule {
    fn check(&self, item: &AlertRule) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRuleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AlertRuleRule for AlertRuleAllowedCodeRule {
    fn check(&self, item: &AlertRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRuleRequiredKeyIdRule {
    pub message: String,
}

impl AlertRuleRule for AlertRuleRequiredKeyIdRule {
    fn check(&self, item: &AlertRule) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRuleArchivedGuardRule {
    pub limit: f64,
}

impl AlertRuleRule for AlertRuleArchivedGuardRule {
    fn check(&self, item: &AlertRule) -> Vec<String> {
        if item.archived && (item.maximum > self.limit) {
            return vec![format!(
                "maximum {} exceeds {} while archived",
                item.maximum, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert rule.
#[derive(Debug, Clone)]
pub struct AlertRuleValidator {
    pub rules: Vec<Rc<dyn AlertRuleRule>>,
}

impl AlertRuleValidator {
    pub fn validate(&self, item: &AlertRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AlertRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_rule_validator() -> AlertRuleValidator {
    AlertRuleValidator {
        rules: vec![
            Rc::new(AlertRuleKeyIdLengthRule { max_length: 3600 }) as Rc<dyn AlertRuleRule>,
            Rc::new(AlertRuleRetriesRangeRule { min: 32, max: 86400 }),
            Rc::new(AlertRuleUpdatedAtRangeRule { min: 80, max: 86400 }),
            Rc::new(AlertRuleAllowedCodeRule {
                allowed: vec!["pending".to_string(), "batched".to_string(), "adaptive".to_string()],
            }),
            Rc::new(AlertRuleRequiredKeyIdRule {
                message: "alert rule needs a value".to_string(),
            }),
            Rc::new(AlertRuleArchivedGuardRule { limit: 8.3 }),
        ],
    }
}
