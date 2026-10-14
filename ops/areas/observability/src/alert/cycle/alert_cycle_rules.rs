use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert cycle must pass. Check returns its problems, or none.
pub trait AlertCycleRule: std::fmt::Debug {
    fn check(&self, item: &AlertCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertCycleLabelLengthRule {
    pub max_length: i64,
}

impl AlertCycleRule for AlertCycleLabelLengthRule {
    fn check(&self, item: &AlertCycle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertCycleRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertCycleRule for AlertCycleRevisionRangeRule {
    fn check(&self, item: &AlertCycle) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertCycleCodeLengthRule {
    pub max_length: i64,
}

impl AlertCycleRule for AlertCycleCodeLengthRule {
    fn check(&self, item: &AlertCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert cycle.
#[derive(Debug, Clone)]
pub struct AlertCycleValidator {
    pub rules: Vec<Rc<dyn AlertCycleRule>>,
}

impl AlertCycleValidator {
    pub fn validate(&self, item: &AlertCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AlertCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_cycle_validator() -> AlertCycleValidator {
    AlertCycleValidator {
        rules: vec![
            Rc::new(AlertCycleLabelLengthRule { max_length: 255 }) as Rc<dyn AlertCycleRule>,
            Rc::new(AlertCycleRevisionRangeRule { min: 16, max: 64 }),
            Rc::new(AlertCycleCodeLengthRule { max_length: 120 }),
        ],
    }
}
