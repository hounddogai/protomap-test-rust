use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy ttl policy schedule must pass. Check returns its problems, or none.
pub trait LegacyTtlPolicyScheduleRule: std::fmt::Debug {
    fn check(&self, item: &LegacyTtlPolicySchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTtlPolicyScheduleRequiredNameRule {
    pub message: String,
}

impl LegacyTtlPolicyScheduleRule for LegacyTtlPolicyScheduleRequiredNameRule {
    fn check(&self, item: &LegacyTtlPolicySchedule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTtlPolicyScheduleNameLengthRule {
    pub max_length: i64,
}

impl LegacyTtlPolicyScheduleRule for LegacyTtlPolicyScheduleNameLengthRule {
    fn check(&self, item: &LegacyTtlPolicySchedule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTtlPolicyScheduleRequiredCountryRule {
    pub message: String,
}

impl LegacyTtlPolicyScheduleRule for LegacyTtlPolicyScheduleRequiredCountryRule {
    fn check(&self, item: &LegacyTtlPolicySchedule) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTtlPolicyScheduleCodeLengthRule {
    pub max_length: i64,
}

impl LegacyTtlPolicyScheduleRule for LegacyTtlPolicyScheduleCodeLengthRule {
    fn check(&self, item: &LegacyTtlPolicySchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy ttl policy schedule.
#[derive(Debug, Clone)]
pub struct LegacyTtlPolicyScheduleValidator {
    pub rules: Vec<Rc<dyn LegacyTtlPolicyScheduleRule>>,
}

impl LegacyTtlPolicyScheduleValidator {
    pub fn validate(&self, item: &LegacyTtlPolicySchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyTtlPolicySchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyTtlPolicySchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_ttl_policy_schedule_validator() -> LegacyTtlPolicyScheduleValidator {
    LegacyTtlPolicyScheduleValidator {
        rules: vec![
            Rc::new(LegacyTtlPolicyScheduleRequiredNameRule {
                message: "legacy ttl policy schedule needs an entry".to_string(),
            }) as Rc<dyn LegacyTtlPolicyScheduleRule>,
            Rc::new(LegacyTtlPolicyScheduleNameLengthRule { max_length: 16 }),
            Rc::new(LegacyTtlPolicyScheduleRequiredCountryRule {
                message: "legacy ttl policy schedule needs a value".to_string(),
            }),
            Rc::new(LegacyTtlPolicyScheduleCodeLengthRule { max_length: 255 }),
        ],
    }
}
