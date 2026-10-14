use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary audit event request must pass. Check returns its problems, or none.
pub trait PrimaryAuditEventRequestRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryAuditEventRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryAuditEventRequestRequiredCodeRule {
    pub message: String,
}

impl PrimaryAuditEventRequestRule for PrimaryAuditEventRequestRequiredCodeRule {
    fn check(&self, item: &PrimaryAuditEventRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryAuditEventRequestCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryAuditEventRequestRule for PrimaryAuditEventRequestCodeLengthRule {
    fn check(&self, item: &PrimaryAuditEventRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryAuditEventRequestCodePrefixRule {
    pub prefix: String,
}

impl PrimaryAuditEventRequestRule for PrimaryAuditEventRequestCodePrefixRule {
    fn check(&self, item: &PrimaryAuditEventRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryAuditEventRequestRequiredNameRule {
    pub message: String,
}

impl PrimaryAuditEventRequestRule for PrimaryAuditEventRequestRequiredNameRule {
    fn check(&self, item: &PrimaryAuditEventRequest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryAuditEventRequestWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryAuditEventRequestRule for PrimaryAuditEventRequestWindowDaysRangeRule {
    fn check(&self, item: &PrimaryAuditEventRequest) -> Vec<String> {
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
pub struct PrimaryAuditEventRequestExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryAuditEventRequestRule for PrimaryAuditEventRequestExpiresAtRangeRule {
    fn check(&self, item: &PrimaryAuditEventRequest) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary audit event request.
#[derive(Debug, Clone)]
pub struct PrimaryAuditEventRequestValidator {
    pub rules: Vec<Rc<dyn PrimaryAuditEventRequestRule>>,
}

impl PrimaryAuditEventRequestValidator {
    pub fn validate(&self, item: &PrimaryAuditEventRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryAuditEventRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryAuditEventRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_audit_event_request_validator() -> PrimaryAuditEventRequestValidator {
    PrimaryAuditEventRequestValidator {
        rules: vec![
            Rc::new(PrimaryAuditEventRequestRequiredCodeRule {
                message: "primary audit event request needs an entry".to_string(),
            }) as Rc<dyn PrimaryAuditEventRequestRule>,
            Rc::new(PrimaryAuditEventRequestCodeLengthRule { max_length: 255 }),
            Rc::new(PrimaryAuditEventRequestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PrimaryAuditEventRequestRequiredNameRule {
                message: "primary audit event request needs a value".to_string(),
            }),
            Rc::new(PrimaryAuditEventRequestWindowDaysRangeRule { min: 64, max: 3600 }),
            Rc::new(PrimaryAuditEventRequestExpiresAtRangeRule { min: 16, max: 64 }),
        ],
    }
}
