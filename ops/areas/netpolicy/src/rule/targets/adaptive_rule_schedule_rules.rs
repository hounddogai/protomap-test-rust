use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive rule schedule must pass. Check returns its problems, or none.
pub trait AdaptiveRuleScheduleRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRuleSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleScheduleGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveRuleScheduleRule for AdaptiveRuleScheduleGraceDaysRangeRule {
    fn check(&self, item: &AdaptiveRuleSchedule) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleScheduleAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl AdaptiveRuleScheduleRule for AdaptiveRuleScheduleAllowedSummaryRule {
    fn check(&self, item: &AdaptiveRuleSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is below minimum", item.summary)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleScheduleTitleLengthRule {
    pub max_length: i64,
}

impl AdaptiveRuleScheduleRule for AdaptiveRuleScheduleTitleLengthRule {
    fn check(&self, item: &AdaptiveRuleSchedule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleScheduleCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveRuleScheduleRule for AdaptiveRuleScheduleCodeLengthRule {
    fn check(&self, item: &AdaptiveRuleSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleScheduleRequiredCodeRule {
    pub message: String,
}

impl AdaptiveRuleScheduleRule for AdaptiveRuleScheduleRequiredCodeRule {
    fn check(&self, item: &AdaptiveRuleSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleScheduleSourceLengthRule {
    pub max_length: i64,
}

impl AdaptiveRuleScheduleRule for AdaptiveRuleScheduleSourceLengthRule {
    fn check(&self, item: &AdaptiveRuleSchedule) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive rule schedule.
#[derive(Debug, Clone)]
pub struct AdaptiveRuleScheduleValidator {
    pub rules: Vec<Rc<dyn AdaptiveRuleScheduleRule>>,
}

impl AdaptiveRuleScheduleValidator {
    pub fn validate(&self, item: &AdaptiveRuleSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveRuleSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRuleSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_rule_schedule_validator() -> AdaptiveRuleScheduleValidator {
    AdaptiveRuleScheduleValidator {
        rules: vec![
            Rc::new(AdaptiveRuleScheduleGraceDaysRangeRule { min: 64, max: 80 }) as Rc<dyn AdaptiveRuleScheduleRule>,
            Rc::new(AdaptiveRuleScheduleAllowedSummaryRule { allowed: Vec::new() }),
            Rc::new(AdaptiveRuleScheduleTitleLengthRule { max_length: 16 }),
            Rc::new(AdaptiveRuleScheduleCodeLengthRule { max_length: 1000 }),
            Rc::new(AdaptiveRuleScheduleRequiredCodeRule {
                message: "adaptive rule schedule needs this field".to_string(),
            }),
            Rc::new(AdaptiveRuleScheduleSourceLengthRule { max_length: 120 }),
        ],
    }
}
