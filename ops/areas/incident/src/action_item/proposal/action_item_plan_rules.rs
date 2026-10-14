use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item plan must pass. Check returns its problems, or none.
pub trait ActionItemPlanRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPlanFingerprintLengthRule {
    pub max_length: i64,
}

impl ActionItemPlanRule for ActionItemPlanFingerprintLengthRule {
    fn check(&self, item: &ActionItemPlan) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPlanRequiredNameRule {
    pub message: String,
}

impl ActionItemPlanRule for ActionItemPlanRequiredNameRule {
    fn check(&self, item: &ActionItemPlan) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPlanCodePrefixRule {
    pub prefix: String,
}

impl ActionItemPlanRule for ActionItemPlanCodePrefixRule {
    fn check(&self, item: &ActionItemPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPlanQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemPlanRule for ActionItemPlanQuotaUnitsRangeRule {
    fn check(&self, item: &ActionItemPlan) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPlanActiveGuardRule {
    pub limit: f64,
}

impl ActionItemPlanRule for ActionItemPlanActiveGuardRule {
    fn check(&self, item: &ActionItemPlan) -> Vec<String> {
        if item.active && (item.ratio > self.limit) {
            return vec![format!("ratio {} exceeds {} while active", item.ratio, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item plan.
#[derive(Debug, Clone)]
pub struct ActionItemPlanValidator {
    pub rules: Vec<Rc<dyn ActionItemPlanRule>>,
}

impl ActionItemPlanValidator {
    pub fn validate(&self, item: &ActionItemPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ActionItemPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_plan_validator() -> ActionItemPlanValidator {
    ActionItemPlanValidator {
        rules: vec![
            Rc::new(ActionItemPlanFingerprintLengthRule { max_length: 32 }) as Rc<dyn ActionItemPlanRule>,
            Rc::new(ActionItemPlanRequiredNameRule {
                message: "action item plan needs an entry".to_string(),
            }),
            Rc::new(ActionItemPlanCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ActionItemPlanQuotaUnitsRangeRule { min: 32, max: 255 }),
            Rc::new(ActionItemPlanActiveGuardRule { limit: 5.9 }),
        ],
    }
}
