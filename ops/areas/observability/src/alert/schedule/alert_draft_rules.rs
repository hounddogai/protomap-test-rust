use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert draft must pass. Check returns its problems, or none.
pub trait AlertDraftRule: std::fmt::Debug {
    fn check(&self, item: &AlertDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertDraftWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertDraftRule for AlertDraftWindowDaysRangeRule {
    fn check(&self, item: &AlertDraft) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertDraftLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertDraftRule for AlertDraftLimitRangeRule {
    fn check(&self, item: &AlertDraft) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertDraftBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertDraftRule for AlertDraftBurstRangeRule {
    fn check(&self, item: &AlertDraft) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertDraftUnitLengthRule {
    pub max_length: i64,
}

impl AlertDraftRule for AlertDraftUnitLengthRule {
    fn check(&self, item: &AlertDraft) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertDraftVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertDraftRule for AlertDraftVersionRangeRule {
    fn check(&self, item: &AlertDraft) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a alert draft.
#[derive(Debug, Clone)]
pub struct AlertDraftValidator {
    pub rules: Vec<Rc<dyn AlertDraftRule>>,
}

impl AlertDraftValidator {
    pub fn validate(&self, item: &AlertDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_draft_validator() -> AlertDraftValidator {
    AlertDraftValidator {
        rules: vec![
            Rc::new(AlertDraftWindowDaysRangeRule { min: 32, max: 3600 }) as Rc<dyn AlertDraftRule>,
            Rc::new(AlertDraftLimitRangeRule { min: 64, max: 1000 }),
            Rc::new(AlertDraftBurstRangeRule { min: 32, max: 80 }),
            Rc::new(AlertDraftUnitLengthRule { max_length: 80 }),
            Rc::new(AlertDraftVersionRangeRule { min: 64, max: 3600 }),
        ],
    }
}
