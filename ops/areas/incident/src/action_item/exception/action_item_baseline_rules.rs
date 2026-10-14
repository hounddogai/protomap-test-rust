use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item baseline must pass. Check returns its problems, or none.
pub trait ActionItemBaselineRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBaselineQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemBaselineRule for ActionItemBaselineQuotaUnitsRangeRule {
    fn check(&self, item: &ActionItemBaseline) -> Vec<String> {
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
pub struct ActionItemBaselineCodePrefixRule {
    pub prefix: String,
}

impl ActionItemBaselineRule for ActionItemBaselineCodePrefixRule {
    fn check(&self, item: &ActionItemBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBaselineGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemBaselineRule for ActionItemBaselineGraceDaysRangeRule {
    fn check(&self, item: &ActionItemBaseline) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBaselineRegionLengthRule {
    pub max_length: i64,
}

impl ActionItemBaselineRule for ActionItemBaselineRegionLengthRule {
    fn check(&self, item: &ActionItemBaseline) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item baseline.
#[derive(Debug, Clone)]
pub struct ActionItemBaselineValidator {
    pub rules: Vec<Rc<dyn ActionItemBaselineRule>>,
}

impl ActionItemBaselineValidator {
    pub fn validate(&self, item: &ActionItemBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ActionItemBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_baseline_validator() -> ActionItemBaselineValidator {
    ActionItemBaselineValidator {
        rules: vec![
            Rc::new(ActionItemBaselineQuotaUnitsRangeRule { min: 16, max: 64 }) as Rc<dyn ActionItemBaselineRule>,
            Rc::new(ActionItemBaselineCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ActionItemBaselineGraceDaysRangeRule { min: 16, max: 86400 }),
            Rc::new(ActionItemBaselineRegionLengthRule { max_length: 1000 }),
        ],
    }
}
