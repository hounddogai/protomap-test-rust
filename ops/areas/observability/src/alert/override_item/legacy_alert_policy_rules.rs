use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy alert policy must pass. Check returns its problems, or none.
pub trait LegacyAlertPolicyRule: std::fmt::Debug {
    fn check(&self, item: &LegacyAlertPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAlertPolicyBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyAlertPolicyRule for LegacyAlertPolicyBurstRangeRule {
    fn check(&self, item: &LegacyAlertPolicy) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAlertPolicyCodePrefixRule {
    pub prefix: String,
}

impl LegacyAlertPolicyRule for LegacyAlertPolicyCodePrefixRule {
    fn check(&self, item: &LegacyAlertPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAlertPolicyCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyAlertPolicyRule for LegacyAlertPolicyCreatedAtRangeRule {
    fn check(&self, item: &LegacyAlertPolicy) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAlertPolicyLabelLengthRule {
    pub max_length: i64,
}

impl LegacyAlertPolicyRule for LegacyAlertPolicyLabelLengthRule {
    fn check(&self, item: &LegacyAlertPolicy) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAlertPolicyAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl LegacyAlertPolicyRule for LegacyAlertPolicyAllowedLabelRule {
    fn check(&self, item: &LegacyAlertPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is out of range", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAlertPolicyMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyAlertPolicyRule for LegacyAlertPolicyMinimumRangeRule {
    fn check(&self, item: &LegacyAlertPolicy) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy alert policy.
#[derive(Debug, Clone)]
pub struct LegacyAlertPolicyValidator {
    pub rules: Vec<Rc<dyn LegacyAlertPolicyRule>>,
}

impl LegacyAlertPolicyValidator {
    pub fn validate(&self, item: &LegacyAlertPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyAlertPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyAlertPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_alert_policy_validator() -> LegacyAlertPolicyValidator {
    LegacyAlertPolicyValidator {
        rules: vec![
            Rc::new(LegacyAlertPolicyBurstRangeRule { min: 16, max: 32 }) as Rc<dyn LegacyAlertPolicyRule>,
            Rc::new(LegacyAlertPolicyCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(LegacyAlertPolicyCreatedAtRangeRule { min: 255, max: 255 }),
            Rc::new(LegacyAlertPolicyLabelLengthRule { max_length: 1000 }),
            Rc::new(LegacyAlertPolicyAllowedLabelRule {
                allowed: vec!["global".to_string()],
            }),
            Rc::new(LegacyAlertPolicyMinimumRangeRule { min: 9.5, max: 4164.4 }),
        ],
    }
}
