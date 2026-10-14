use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled slo request must pass. Check returns its problems, or none.
pub trait ScheduledSloRequestRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSloRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSloRequestRequiredNameRule {
    pub message: String,
}

impl ScheduledSloRequestRule for ScheduledSloRequestRequiredNameRule {
    fn check(&self, item: &ScheduledSloRequest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSloRequestGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledSloRequestRule for ScheduledSloRequestGraceDaysRangeRule {
    fn check(&self, item: &ScheduledSloRequest) -> Vec<String> {
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
pub struct ScheduledSloRequestUnitLengthRule {
    pub max_length: i64,
}

impl ScheduledSloRequestRule for ScheduledSloRequestUnitLengthRule {
    fn check(&self, item: &ScheduledSloRequest) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSloRequestCodePrefixRule {
    pub prefix: String,
}

impl ScheduledSloRequestRule for ScheduledSloRequestCodePrefixRule {
    fn check(&self, item: &ScheduledSloRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSloRequestExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledSloRequestRule for ScheduledSloRequestExpiresAtRangeRule {
    fn check(&self, item: &ScheduledSloRequest) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSloRequestActiveGuardRule {
    pub limit: i64,
}

impl ScheduledSloRequestRule for ScheduledSloRequestActiveGuardRule {
    fn check(&self, item: &ScheduledSloRequest) -> Vec<String> {
        if item.active && (item.grace_days > self.limit) {
            return vec![format!(
                "grace days {} exceeds {} while active",
                item.grace_days, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled slo request.
#[derive(Debug, Clone)]
pub struct ScheduledSloRequestValidator {
    pub rules: Vec<Rc<dyn ScheduledSloRequestRule>>,
}

impl ScheduledSloRequestValidator {
    pub fn validate(&self, item: &ScheduledSloRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledSloRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSloRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_slo_request_validator() -> ScheduledSloRequestValidator {
    ScheduledSloRequestValidator {
        rules: vec![
            Rc::new(ScheduledSloRequestRequiredNameRule {
                message: "scheduled slo request needs this field".to_string(),
            }) as Rc<dyn ScheduledSloRequestRule>,
            Rc::new(ScheduledSloRequestGraceDaysRangeRule { min: 16, max: 64 }),
            Rc::new(ScheduledSloRequestUnitLengthRule { max_length: 120 }),
            Rc::new(ScheduledSloRequestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ScheduledSloRequestExpiresAtRangeRule { min: 16, max: 86400 }),
            Rc::new(ScheduledSloRequestActiveGuardRule { limit: 255 }),
        ],
    }
}
