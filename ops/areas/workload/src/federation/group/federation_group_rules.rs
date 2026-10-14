use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation group must pass. Check returns its problems, or none.
pub trait FederationGroupRule: std::fmt::Debug {
    fn check(&self, item: &FederationGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationGroupRequiredCodeRule {
    pub message: String,
}

impl FederationGroupRule for FederationGroupRequiredCodeRule {
    fn check(&self, item: &FederationGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationGroupTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl FederationGroupRule for FederationGroupTagsLimitRule {
    fn check(&self, item: &FederationGroup) -> Vec<String> {
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
pub struct FederationGroupRequiredTimezoneRule {
    pub message: String,
}

impl FederationGroupRule for FederationGroupRequiredTimezoneRule {
    fn check(&self, item: &FederationGroup) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationGroupBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FederationGroupRule for FederationGroupBudgetRangeRule {
    fn check(&self, item: &FederationGroup) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationGroupAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FederationGroupRule for FederationGroupAmountRangeRule {
    fn check(&self, item: &FederationGroup) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a federation group.
#[derive(Debug, Clone)]
pub struct FederationGroupValidator {
    pub rules: Vec<Rc<dyn FederationGroupRule>>,
}

impl FederationGroupValidator {
    pub fn validate(&self, item: &FederationGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FederationGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_group_validator() -> FederationGroupValidator {
    FederationGroupValidator {
        rules: vec![
            Rc::new(FederationGroupRequiredCodeRule {
                message: "federation group needs a value".to_string(),
            }) as Rc<dyn FederationGroupRule>,
            Rc::new(FederationGroupTagsLimitRule {
                max_count: 120,
                banned: vec!["weighted".to_string(), "legacy".to_string(), "seasonal".to_string()],
            }),
            Rc::new(FederationGroupRequiredTimezoneRule {
                message: "federation group needs an entry".to_string(),
            }),
            Rc::new(FederationGroupBudgetRangeRule { min: 6.4, max: 4970.6 }),
            Rc::new(FederationGroupAmountRangeRule { min: 5.2, max: 4618.5 }),
        ],
    }
}
