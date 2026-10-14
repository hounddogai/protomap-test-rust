use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover preference must pass. Check returns its problems, or none.
pub trait FailoverPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &FailoverPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverPreferenceWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverPreferenceRule for FailoverPreferenceWindowDaysRangeRule {
    fn check(&self, item: &FailoverPreference) -> Vec<String> {
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
pub struct FailoverPreferenceRequiredTimezoneRule {
    pub message: String,
}

impl FailoverPreferenceRule for FailoverPreferenceRequiredTimezoneRule {
    fn check(&self, item: &FailoverPreference) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverPreferenceGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverPreferenceRule for FailoverPreferenceGraceDaysRangeRule {
    fn check(&self, item: &FailoverPreference) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverPreferenceRequiredContactEmailRule {
    pub message: String,
}

impl FailoverPreferenceRule for FailoverPreferenceRequiredContactEmailRule {
    fn check(&self, item: &FailoverPreference) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover preference.
#[derive(Debug, Clone)]
pub struct FailoverPreferenceValidator {
    pub rules: Vec<Rc<dyn FailoverPreferenceRule>>,
}

impl FailoverPreferenceValidator {
    pub fn validate(&self, item: &FailoverPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_preference_validator() -> FailoverPreferenceValidator {
    FailoverPreferenceValidator {
        rules: vec![
            Rc::new(FailoverPreferenceWindowDaysRangeRule { min: 16, max: 86400 }) as Rc<dyn FailoverPreferenceRule>,
            Rc::new(FailoverPreferenceRequiredTimezoneRule {
                message: "failover preference needs an entry".to_string(),
            }),
            Rc::new(FailoverPreferenceGraceDaysRangeRule { min: 120, max: 255 }),
            Rc::new(FailoverPreferenceRequiredContactEmailRule {
                message: "failover preference needs an entry".to_string(),
            }),
        ],
    }
}
