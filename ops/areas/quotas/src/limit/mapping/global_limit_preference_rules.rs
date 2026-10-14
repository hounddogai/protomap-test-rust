use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global limit preference must pass. Check returns its problems, or none.
pub trait GlobalLimitPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &GlobalLimitPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitPreferenceRequiredTitleRule {
    pub message: String,
}

impl GlobalLimitPreferenceRule for GlobalLimitPreferenceRequiredTitleRule {
    fn check(&self, item: &GlobalLimitPreference) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitPreferenceCodePrefixRule {
    pub prefix: String,
}

impl GlobalLimitPreferenceRule for GlobalLimitPreferenceCodePrefixRule {
    fn check(&self, item: &GlobalLimitPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitPreferenceRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalLimitPreferenceRule for GlobalLimitPreferenceRotationDaysRangeRule {
    fn check(&self, item: &GlobalLimitPreference) -> Vec<String> {
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

/// Runs every rule of a global limit preference.
#[derive(Debug, Clone)]
pub struct GlobalLimitPreferenceValidator {
    pub rules: Vec<Rc<dyn GlobalLimitPreferenceRule>>,
}

impl GlobalLimitPreferenceValidator {
    pub fn validate(&self, item: &GlobalLimitPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalLimitPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalLimitPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_limit_preference_validator() -> GlobalLimitPreferenceValidator {
    GlobalLimitPreferenceValidator {
        rules: vec![
            Rc::new(GlobalLimitPreferenceRequiredTitleRule {
                message: "global limit preference needs this field".to_string(),
            }) as Rc<dyn GlobalLimitPreferenceRule>,
            Rc::new(GlobalLimitPreferenceCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(GlobalLimitPreferenceRotationDaysRangeRule { min: 64, max: 1000 }),
        ],
    }
}
