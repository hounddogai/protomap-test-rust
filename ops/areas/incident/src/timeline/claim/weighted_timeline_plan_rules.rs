use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted timeline plan must pass. Check returns its problems, or none.
pub trait WeightedTimelinePlanRule: std::fmt::Debug {
    fn check(&self, item: &WeightedTimelinePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelinePlanRequiredTimezoneRule {
    pub message: String,
}

impl WeightedTimelinePlanRule for WeightedTimelinePlanRequiredTimezoneRule {
    fn check(&self, item: &WeightedTimelinePlan) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelinePlanCodePrefixRule {
    pub prefix: String,
}

impl WeightedTimelinePlanRule for WeightedTimelinePlanCodePrefixRule {
    fn check(&self, item: &WeightedTimelinePlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelinePlanTimezoneLengthRule {
    pub max_length: i64,
}

impl WeightedTimelinePlanRule for WeightedTimelinePlanTimezoneLengthRule {
    fn check(&self, item: &WeightedTimelinePlan) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelinePlanLabelLengthRule {
    pub max_length: i64,
}

impl WeightedTimelinePlanRule for WeightedTimelinePlanLabelLengthRule {
    fn check(&self, item: &WeightedTimelinePlan) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelinePlanAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedTimelinePlanRule for WeightedTimelinePlanAttemptsRangeRule {
    fn check(&self, item: &WeightedTimelinePlan) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted timeline plan.
#[derive(Debug, Clone)]
pub struct WeightedTimelinePlanValidator {
    pub rules: Vec<Rc<dyn WeightedTimelinePlanRule>>,
}

impl WeightedTimelinePlanValidator {
    pub fn validate(&self, item: &WeightedTimelinePlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedTimelinePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedTimelinePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_timeline_plan_validator() -> WeightedTimelinePlanValidator {
    WeightedTimelinePlanValidator {
        rules: vec![
            Rc::new(WeightedTimelinePlanRequiredTimezoneRule {
                message: "weighted timeline plan needs this field".to_string(),
            }) as Rc<dyn WeightedTimelinePlanRule>,
            Rc::new(WeightedTimelinePlanCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(WeightedTimelinePlanTimezoneLengthRule { max_length: 120 }),
            Rc::new(WeightedTimelinePlanLabelLengthRule { max_length: 80 }),
            Rc::new(WeightedTimelinePlanAttemptsRangeRule { min: 32, max: 1000 }),
        ],
    }
}
