use super::*;

use acme_kit::text;

/// Where a audit event target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventTargetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AuditEventTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AuditEventTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventTarget {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub ttl_seconds: i64,
    pub checksum: String,
    pub approved: bool,
    pub cluster: String,
    pub status: AuditEventTargetStatus,
}

impl AuditEventTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventTargetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventTargetStatus) -> bool {
        match self.status {
            AuditEventTargetStatus::Proposed => {
                match next {
                    AuditEventTargetStatus::Approved | AuditEventTargetStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventTargetStatus::Approved => {
                match next {
                    AuditEventTargetStatus::Rejected | AuditEventTargetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventTargetStatus::Rejected => {
                match next {
                    AuditEventTargetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "audit event target", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> AuditEventTarget {
        AuditEventTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            ttl_seconds: self.ttl_seconds,
            checksum: self.checksum.clone(),
            approved: self.approved,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a audit event target in status proposed.
pub fn new_audit_event_target(code: &str, label: &str) -> AuditEventTarget {
    AuditEventTarget {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        ttl_seconds: 0,
        checksum: "".to_string(),
        approved: false,
        cluster: "".to_string(),
        status: AuditEventTargetStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_audit_event_target("code-163", "label-357");
        assert_eq!(item.status, AuditEventTargetStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_audit_event_target("code-163", "label-357");
        if item.move_to(AuditEventTargetStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventTargetStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_audit_event_target("code-163", "label-357");
        item.status = AuditEventTargetStatus::Withdrawn;
        assert!(
            item.move_to(AuditEventTargetStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
