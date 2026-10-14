use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a allowlist schedule must pass. Check returns its problems, or none.
pub trait AllowlistScheduleRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistScheduleBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistScheduleRule for AllowlistScheduleBatchSizeRangeRule {
    fn check(&self, item: &AllowlistSchedule) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistScheduleRequiredChannelRule {
    pub message: String,
}

impl AllowlistScheduleRule for AllowlistScheduleRequiredChannelRule {
    fn check(&self, item: &AllowlistSchedule) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistScheduleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AllowlistScheduleRule for AllowlistScheduleAllowedCodeRule {
    fn check(&self, item: &AllowlistSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist schedule.
#[derive(Debug, Clone)]
pub struct AllowlistScheduleValidator {
    pub rules: Vec<Rc<dyn AllowlistScheduleRule>>,
}

impl AllowlistScheduleValidator {
    pub fn validate(&self, item: &AllowlistSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_schedule_validator() -> AllowlistScheduleValidator {
    AllowlistScheduleValidator {
        rules: vec![
            Rc::new(AllowlistScheduleBatchSizeRangeRule { min: 16, max: 3600 }) as Rc<dyn AllowlistScheduleRule>,
            Rc::new(AllowlistScheduleRequiredChannelRule {
                message: "allowlist schedule needs an entry".to_string(),
            }),
            Rc::new(AllowlistScheduleAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}
