use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a audit event policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventPolicyStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AuditEventPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AuditEventPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One audit event policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPolicy {
    pub code: String,
    pub name: String,
    pub title: String,
    pub ceiling: f64,
    pub minimum: f64,
    pub scope: String,
    pub channel: String,
    pub rotation_days: i64,
    pub principal: String,
    pub ttl_seconds: i64,
    pub attributes: BTreeMap<String, String>,
    pub status: AuditEventPolicyStatus,
}

impl AuditEventPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventPolicyStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventPolicyStatus) -> bool {
        match self.status {
            AuditEventPolicyStatus::Proposed => {
                return next == AuditEventPolicyStatus::Rejected;
            }
            AuditEventPolicyStatus::Approved => {
                return next == AuditEventPolicyStatus::Rejected || next == AuditEventPolicyStatus::Withdrawn;
            }
            AuditEventPolicyStatus::Rejected => {
                return next == AuditEventPolicyStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.minimum * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventPolicy {
        AuditEventPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            minimum: self.minimum,
            scope: self.scope.clone(),
            channel: self.channel.clone(),
            rotation_days: self.rotation_days,
            principal: self.principal.clone(),
            ttl_seconds: self.ttl_seconds,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a audit event policy in status proposed.
pub fn new_audit_event_policy(code: &str, name: &str) -> AuditEventPolicy {
    AuditEventPolicy {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        ceiling: 0.0,
        minimum: 0.0,
        scope: "".to_string(),
        channel: "".to_string(),
        rotation_days: 0,
        principal: "".to_string(),
        ttl_seconds: 0,
        attributes: BTreeMap::new(),
        status: AuditEventPolicyStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_audit_event_policy("code-202", "name-699");
        assert_eq!(item.status, AuditEventPolicyStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_audit_event_policy("code-202", "name-699");
        if item.move_to(AuditEventPolicyStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventPolicyStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_audit_event_policy("code-202", "name-699");
        item.status = AuditEventPolicyStatus::Withdrawn;
        assert!(
            item.move_to(AuditEventPolicyStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
