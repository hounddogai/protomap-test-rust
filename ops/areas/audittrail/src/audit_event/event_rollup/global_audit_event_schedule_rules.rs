use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global audit event schedule must pass. Check returns its problems, or none.
pub trait GlobalAuditEventScheduleRule: std::fmt::Debug {
    fn check(&self, item: &GlobalAuditEventSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventScheduleIssuerLengthRule {
    pub max_length: i64,
}

impl GlobalAuditEventScheduleRule for GlobalAuditEventScheduleIssuerLengthRule {
    fn check(&self, item: &GlobalAuditEventSchedule) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventScheduleStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalAuditEventScheduleRule for GlobalAuditEventScheduleStartsAtRangeRule {
    fn check(&self, item: &GlobalAuditEventSchedule) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventScheduleRequiredIssuerRule {
    pub message: String,
}

impl GlobalAuditEventScheduleRule for GlobalAuditEventScheduleRequiredIssuerRule {
    fn check(&self, item: &GlobalAuditEventSchedule) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventScheduleArchivedGuardRule {
    pub limit: i64,
}

impl GlobalAuditEventScheduleRule for GlobalAuditEventScheduleArchivedGuardRule {
    fn check(&self, item: &GlobalAuditEventSchedule) -> Vec<String> {
        if item.archived && (item.starts_at > self.limit) {
            return vec![format!(
                "starts at {} exceeds {} while archived",
                item.starts_at, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a global audit event schedule.
#[derive(Debug, Clone)]
pub struct GlobalAuditEventScheduleValidator {
    pub rules: Vec<Rc<dyn GlobalAuditEventScheduleRule>>,
}

impl GlobalAuditEventScheduleValidator {
    pub fn validate(&self, item: &GlobalAuditEventSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalAuditEventSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalAuditEventSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_audit_event_schedule_validator() -> GlobalAuditEventScheduleValidator {
    GlobalAuditEventScheduleValidator {
        rules: vec![
            Rc::new(GlobalAuditEventScheduleIssuerLengthRule { max_length: 1000 })
                as Rc<dyn GlobalAuditEventScheduleRule>,
            Rc::new(GlobalAuditEventScheduleStartsAtRangeRule { min: 64, max: 3600 }),
            Rc::new(GlobalAuditEventScheduleRequiredIssuerRule {
                message: "global audit event schedule needs an entry".to_string(),
            }),
            Rc::new(GlobalAuditEventScheduleArchivedGuardRule { limit: 16 }),
        ],
    }
}
