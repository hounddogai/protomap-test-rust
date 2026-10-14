use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a audit event quota must pass. Check returns its problems, or none.
pub trait AuditEventQuotaRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventQuotaAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl AuditEventQuotaRule for AuditEventQuotaAllowedSegmentRule {
    fn check(&self, item: &AuditEventQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is too long", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventQuotaRequiredSegmentRule {
    pub message: String,
}

impl AuditEventQuotaRule for AuditEventQuotaRequiredSegmentRule {
    fn check(&self, item: &AuditEventQuota) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventQuotaApprovedGuardRule {
    pub limit: i64,
}

impl AuditEventQuotaRule for AuditEventQuotaApprovedGuardRule {
    fn check(&self, item: &AuditEventQuota) -> Vec<String> {
        if item.approved && (item.grace_days > self.limit) {
            return vec![format!(
                "grace days {} exceeds {} while approved",
                item.grace_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventQuotaRequiredLabelRule {
    pub message: String,
}

impl AuditEventQuotaRule for AuditEventQuotaRequiredLabelRule {
    fn check(&self, item: &AuditEventQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event quota.
#[derive(Debug, Clone)]
pub struct AuditEventQuotaValidator {
    pub rules: Vec<Rc<dyn AuditEventQuotaRule>>,
}

impl AuditEventQuotaValidator {
    pub fn validate(&self, item: &AuditEventQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_quota_validator() -> AuditEventQuotaValidator {
    AuditEventQuotaValidator {
        rules: vec![
            Rc::new(AuditEventQuotaAllowedSegmentRule { allowed: Vec::new() }) as Rc<dyn AuditEventQuotaRule>,
            Rc::new(AuditEventQuotaRequiredSegmentRule {
                message: "audit event quota needs an entry".to_string(),
            }),
            Rc::new(AuditEventQuotaApprovedGuardRule { limit: 3600 }),
            Rc::new(AuditEventQuotaRequiredLabelRule {
                message: "audit event quota needs this field".to_string(),
            }),
        ],
    }
}
