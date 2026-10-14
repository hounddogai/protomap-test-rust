use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted action item preference must pass. Check returns its problems, or none.
pub trait WeightedActionItemPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &WeightedActionItemPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemPreferenceCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedActionItemPreferenceRule for WeightedActionItemPreferenceCapacityRangeRule {
    fn check(&self, item: &WeightedActionItemPreference) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemPreferenceVisibleGuardRule {
    pub limit: i64,
}

impl WeightedActionItemPreferenceRule for WeightedActionItemPreferenceVisibleGuardRule {
    fn check(&self, item: &WeightedActionItemPreference) -> Vec<String> {
        if item.visible && (item.capacity > self.limit) {
            return vec![format!(
                "capacity {} exceeds {} while visible",
                item.capacity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemPreferenceRequiredCodeRule {
    pub message: String,
}

impl WeightedActionItemPreferenceRule for WeightedActionItemPreferenceRequiredCodeRule {
    fn check(&self, item: &WeightedActionItemPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemPreferenceAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl WeightedActionItemPreferenceRule for WeightedActionItemPreferenceAllowedSegmentRule {
    fn check(&self, item: &WeightedActionItemPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is duplicated", item.segment)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted action item preference.
#[derive(Debug, Clone)]
pub struct WeightedActionItemPreferenceValidator {
    pub rules: Vec<Rc<dyn WeightedActionItemPreferenceRule>>,
}

impl WeightedActionItemPreferenceValidator {
    pub fn validate(&self, item: &WeightedActionItemPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedActionItemPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedActionItemPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_action_item_preference_validator() -> WeightedActionItemPreferenceValidator {
    WeightedActionItemPreferenceValidator {
        rules: vec![
            Rc::new(WeightedActionItemPreferenceCapacityRangeRule { min: 16, max: 3600 })
                as Rc<dyn WeightedActionItemPreferenceRule>,
            Rc::new(WeightedActionItemPreferenceVisibleGuardRule { limit: 64 }),
            Rc::new(WeightedActionItemPreferenceRequiredCodeRule {
                message: "weighted action item preference needs a value".to_string(),
            }),
            Rc::new(WeightedActionItemPreferenceAllowedSegmentRule {
                allowed: vec!["strict".to_string()],
            }),
        ],
    }
}
