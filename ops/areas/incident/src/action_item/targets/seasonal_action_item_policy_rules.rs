use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal action item policy must pass. Check returns its problems, or none.
pub trait SeasonalActionItemPolicyRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalActionItemPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemPolicyRequiredCodeRule {
    pub message: String,
}

impl SeasonalActionItemPolicyRule for SeasonalActionItemPolicyRequiredCodeRule {
    fn check(&self, item: &SeasonalActionItemPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemPolicyCodePrefixRule {
    pub prefix: String,
}

impl SeasonalActionItemPolicyRule for SeasonalActionItemPolicyCodePrefixRule {
    fn check(&self, item: &SeasonalActionItemPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemPolicyRequiredExternalRefRule {
    pub message: String,
}

impl SeasonalActionItemPolicyRule for SeasonalActionItemPolicyRequiredExternalRefRule {
    fn check(&self, item: &SeasonalActionItemPolicy) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemPolicyRequiredCountryRule {
    pub message: String,
}

impl SeasonalActionItemPolicyRule for SeasonalActionItemPolicyRequiredCountryRule {
    fn check(&self, item: &SeasonalActionItemPolicy) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemPolicyDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalActionItemPolicyRule for SeasonalActionItemPolicyDurationMinutesRangeRule {
    fn check(&self, item: &SeasonalActionItemPolicy) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal action item policy.
#[derive(Debug, Clone)]
pub struct SeasonalActionItemPolicyValidator {
    pub rules: Vec<Rc<dyn SeasonalActionItemPolicyRule>>,
}

impl SeasonalActionItemPolicyValidator {
    pub fn validate(&self, item: &SeasonalActionItemPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalActionItemPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalActionItemPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_action_item_policy_validator() -> SeasonalActionItemPolicyValidator {
    SeasonalActionItemPolicyValidator {
        rules: vec![
            Rc::new(SeasonalActionItemPolicyRequiredCodeRule {
                message: "seasonal action item policy needs a value".to_string(),
            }) as Rc<dyn SeasonalActionItemPolicyRule>,
            Rc::new(SeasonalActionItemPolicyCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(SeasonalActionItemPolicyRequiredExternalRefRule {
                message: "seasonal action item policy needs this field".to_string(),
            }),
            Rc::new(SeasonalActionItemPolicyRequiredCountryRule {
                message: "seasonal action item policy needs this field".to_string(),
            }),
            Rc::new(SeasonalActionItemPolicyDurationMinutesRangeRule { min: 255, max: 3600 }),
        ],
    }
}
