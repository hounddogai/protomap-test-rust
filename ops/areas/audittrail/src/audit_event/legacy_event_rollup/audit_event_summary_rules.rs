use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a audit event summary must pass. Check returns its problems, or none.
pub trait AuditEventSummaryRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSummaryPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventSummaryRule for AuditEventSummaryPriorityRangeRule {
    fn check(&self, item: &AuditEventSummary) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSummaryRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AuditEventSummaryRule for AuditEventSummaryRateRangeRule {
    fn check(&self, item: &AuditEventSummary) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSummaryRequiredKeyIdRule {
    pub message: String,
}

impl AuditEventSummaryRule for AuditEventSummaryRequiredKeyIdRule {
    fn check(&self, item: &AuditEventSummary) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSummaryRequiredPhoneRule {
    pub message: String,
}

impl AuditEventSummaryRule for AuditEventSummaryRequiredPhoneRule {
    fn check(&self, item: &AuditEventSummary) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSummaryAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl AuditEventSummaryRule for AuditEventSummaryAllowedPhoneRule {
    fn check(&self, item: &AuditEventSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is inconsistent", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSummaryStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventSummaryRule for AuditEventSummaryStartsAtRangeRule {
    fn check(&self, item: &AuditEventSummary) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event summary.
#[derive(Debug, Clone)]
pub struct AuditEventSummaryValidator {
    pub rules: Vec<Rc<dyn AuditEventSummaryRule>>,
}

impl AuditEventSummaryValidator {
    pub fn validate(&self, item: &AuditEventSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_summary_validator() -> AuditEventSummaryValidator {
    AuditEventSummaryValidator {
        rules: vec![
            Rc::new(AuditEventSummaryPriorityRangeRule { min: 255, max: 1000 }) as Rc<dyn AuditEventSummaryRule>,
            Rc::new(AuditEventSummaryRateRangeRule { min: 4.0, max: 2376.0 }),
            Rc::new(AuditEventSummaryRequiredKeyIdRule {
                message: "audit event summary needs a value".to_string(),
            }),
            Rc::new(AuditEventSummaryRequiredPhoneRule {
                message: "audit event summary needs this field".to_string(),
            }),
            Rc::new(AuditEventSummaryAllowedPhoneRule { allowed: Vec::new() }),
            Rc::new(AuditEventSummaryStartsAtRangeRule { min: 1000, max: 3600 }),
        ],
    }
}
