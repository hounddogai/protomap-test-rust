use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy rule preference must pass. Check returns its problems, or none.
pub trait LegacyRulePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &LegacyRulePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRulePreferenceRequiredNameRule {
    pub message: String,
}

impl LegacyRulePreferenceRule for LegacyRulePreferenceRequiredNameRule {
    fn check(&self, item: &LegacyRulePreference) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRulePreferenceApprovedGuardRule {
    pub limit: i64,
}

impl LegacyRulePreferenceRule for LegacyRulePreferenceApprovedGuardRule {
    fn check(&self, item: &LegacyRulePreference) -> Vec<String> {
        if item.approved && (item.port > self.limit) {
            return vec![format!("port {} exceeds {} while approved", item.port, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRulePreferenceNameLengthRule {
    pub max_length: i64,
}

impl LegacyRulePreferenceRule for LegacyRulePreferenceNameLengthRule {
    fn check(&self, item: &LegacyRulePreference) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRulePreferenceRequiredCodeRule {
    pub message: String,
}

impl LegacyRulePreferenceRule for LegacyRulePreferenceRequiredCodeRule {
    fn check(&self, item: &LegacyRulePreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRulePreferenceWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyRulePreferenceRule for LegacyRulePreferenceWindowDaysRangeRule {
    fn check(&self, item: &LegacyRulePreference) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy rule preference.
#[derive(Debug, Clone)]
pub struct LegacyRulePreferenceValidator {
    pub rules: Vec<Rc<dyn LegacyRulePreferenceRule>>,
}

impl LegacyRulePreferenceValidator {
    pub fn validate(&self, item: &LegacyRulePreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyRulePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyRulePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_rule_preference_validator() -> LegacyRulePreferenceValidator {
    LegacyRulePreferenceValidator {
        rules: vec![
            Rc::new(LegacyRulePreferenceRequiredNameRule {
                message: "legacy rule preference needs an entry".to_string(),
            }) as Rc<dyn LegacyRulePreferenceRule>,
            Rc::new(LegacyRulePreferenceApprovedGuardRule { limit: 86400 }),
            Rc::new(LegacyRulePreferenceNameLengthRule { max_length: 86400 }),
            Rc::new(LegacyRulePreferenceRequiredCodeRule {
                message: "legacy rule preference needs a value".to_string(),
            }),
            Rc::new(LegacyRulePreferenceWindowDaysRangeRule { min: 64, max: 80 }),
        ],
    }
}
