use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending audit event slot must pass. Check returns its problems, or none.
pub trait PendingAuditEventSlotRule: std::fmt::Debug {
    fn check(&self, item: &PendingAuditEventSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAuditEventSlotNameLengthRule {
    pub max_length: i64,
}

impl PendingAuditEventSlotRule for PendingAuditEventSlotNameLengthRule {
    fn check(&self, item: &PendingAuditEventSlot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAuditEventSlotRequiredNameRule {
    pub message: String,
}

impl PendingAuditEventSlotRule for PendingAuditEventSlotRequiredNameRule {
    fn check(&self, item: &PendingAuditEventSlot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAuditEventSlotAllowedNameRule {
    pub allowed: Vec<String>,
}

impl PendingAuditEventSlotRule for PendingAuditEventSlotAllowedNameRule {
    fn check(&self, item: &PendingAuditEventSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is blocked", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAuditEventSlotArchivedGuardRule {
    pub limit: i64,
}

impl PendingAuditEventSlotRule for PendingAuditEventSlotArchivedGuardRule {
    fn check(&self, item: &PendingAuditEventSlot) -> Vec<String> {
        if item.archived && (item.batch_size > self.limit) {
            return vec![format!(
                "batch size {} exceeds {} while archived",
                item.batch_size, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending audit event slot.
#[derive(Debug, Clone)]
pub struct PendingAuditEventSlotValidator {
    pub rules: Vec<Rc<dyn PendingAuditEventSlotRule>>,
}

impl PendingAuditEventSlotValidator {
    pub fn validate(&self, item: &PendingAuditEventSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingAuditEventSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingAuditEventSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_audit_event_slot_validator() -> PendingAuditEventSlotValidator {
    PendingAuditEventSlotValidator {
        rules: vec![
            Rc::new(PendingAuditEventSlotNameLengthRule { max_length: 80 }) as Rc<dyn PendingAuditEventSlotRule>,
            Rc::new(PendingAuditEventSlotRequiredNameRule {
                message: "pending audit event slot needs a value".to_string(),
            }),
            Rc::new(PendingAuditEventSlotAllowedNameRule {
                allowed: vec!["rolling".to_string(), "lenient".to_string()],
            }),
            Rc::new(PendingAuditEventSlotArchivedGuardRule { limit: 3600 }),
        ],
    }
}
