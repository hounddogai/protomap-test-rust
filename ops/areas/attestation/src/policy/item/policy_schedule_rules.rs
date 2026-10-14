use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy schedule must pass. Check returns its problems, or none.
pub trait PolicyScheduleRule: std::fmt::Debug {
    fn check(&self, item: &PolicySchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyScheduleTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl PolicyScheduleRule for PolicyScheduleTagsLimitRule {
    fn check(&self, item: &PolicySchedule) -> Vec<String> {
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
pub struct PolicyScheduleRequiredLabelRule {
    pub message: String,
}

impl PolicyScheduleRule for PolicyScheduleRequiredLabelRule {
    fn check(&self, item: &PolicySchedule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyScheduleRequiredCodeRule {
    pub message: String,
}

impl PolicyScheduleRule for PolicyScheduleRequiredCodeRule {
    fn check(&self, item: &PolicySchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyScheduleRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PolicyScheduleRule for PolicyScheduleRotationDaysRangeRule {
    fn check(&self, item: &PolicySchedule) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyScheduleRequiredSourceRule {
    pub message: String,
}

impl PolicyScheduleRule for PolicyScheduleRequiredSourceRule {
    fn check(&self, item: &PolicySchedule) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy schedule.
#[derive(Debug, Clone)]
pub struct PolicyScheduleValidator {
    pub rules: Vec<Rc<dyn PolicyScheduleRule>>,
}

impl PolicyScheduleValidator {
    pub fn validate(&self, item: &PolicySchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PolicySchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicySchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_schedule_validator() -> PolicyScheduleValidator {
    PolicyScheduleValidator {
        rules: vec![
            Rc::new(PolicyScheduleTagsLimitRule {
                max_count: 120,
                banned: vec!["capped".to_string()],
            }) as Rc<dyn PolicyScheduleRule>,
            Rc::new(PolicyScheduleRequiredLabelRule {
                message: "policy schedule needs an entry".to_string(),
            }),
            Rc::new(PolicyScheduleRequiredCodeRule {
                message: "policy schedule needs this field".to_string(),
            }),
            Rc::new(PolicyScheduleRotationDaysRangeRule { min: 1000, max: 3600 }),
            Rc::new(PolicyScheduleRequiredSourceRule {
                message: "policy schedule needs this field".to_string(),
            }),
        ],
    }
}
