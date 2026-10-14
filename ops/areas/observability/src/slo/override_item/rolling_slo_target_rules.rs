use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling slo target must pass. Check returns its problems, or none.
pub trait RollingSloTargetRule: std::fmt::Debug {
    fn check(&self, item: &RollingSloTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSloTargetNameLengthRule {
    pub max_length: i64,
}

impl RollingSloTargetRule for RollingSloTargetNameLengthRule {
    fn check(&self, item: &RollingSloTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSloTargetFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingSloTargetRule for RollingSloTargetFloorValueRangeRule {
    fn check(&self, item: &RollingSloTarget) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSloTargetTitleLengthRule {
    pub max_length: i64,
}

impl RollingSloTargetRule for RollingSloTargetTitleLengthRule {
    fn check(&self, item: &RollingSloTarget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSloTargetAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl RollingSloTargetRule for RollingSloTargetAllowedNotesRule {
    fn check(&self, item: &RollingSloTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is duplicated", item.notes)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSloTargetRequiredCodeRule {
    pub message: String,
}

impl RollingSloTargetRule for RollingSloTargetRequiredCodeRule {
    fn check(&self, item: &RollingSloTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling slo target.
#[derive(Debug, Clone)]
pub struct RollingSloTargetValidator {
    pub rules: Vec<Rc<dyn RollingSloTargetRule>>,
}

impl RollingSloTargetValidator {
    pub fn validate(&self, item: &RollingSloTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingSloTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingSloTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_slo_target_validator() -> RollingSloTargetValidator {
    RollingSloTargetValidator {
        rules: vec![
            Rc::new(RollingSloTargetNameLengthRule { max_length: 64 }) as Rc<dyn RollingSloTargetRule>,
            Rc::new(RollingSloTargetFloorValueRangeRule { min: 8.1, max: 1751.5 }),
            Rc::new(RollingSloTargetTitleLengthRule { max_length: 32 }),
            Rc::new(RollingSloTargetAllowedNotesRule {
                allowed: vec!["legacy".to_string()],
            }),
            Rc::new(RollingSloTargetRequiredCodeRule {
                message: "rolling slo target needs an entry".to_string(),
            }),
        ],
    }
}
