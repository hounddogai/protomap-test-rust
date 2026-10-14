use super::*;

use acme_kit::text;

/// Where a audit event snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventSnapshotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AuditEventSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AuditEventSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSnapshot {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub priority: i64,
    pub ratio: f64,
    pub revision: i64,
    pub issuer: String,
    pub status: AuditEventSnapshotStatus,
}

impl AuditEventSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventSnapshotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventSnapshotStatus) -> bool {
        match self.status {
            AuditEventSnapshotStatus::Proposed => {
                return next == AuditEventSnapshotStatus::Approved || next == AuditEventSnapshotStatus::Rejected;
            }
            AuditEventSnapshotStatus::Approved => {
                match next {
                    AuditEventSnapshotStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventSnapshotStatus::Rejected => {
                match next {
                    AuditEventSnapshotStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AuditEventSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "audit event snapshot", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.priority as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventSnapshot {
        AuditEventSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            priority: self.priority,
            ratio: self.ratio,
            revision: self.revision,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a audit event snapshot in status proposed.
pub fn new_audit_event_snapshot(code: &str, label: &str) -> AuditEventSnapshot {
    AuditEventSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        priority: 0,
        ratio: 0.0,
        revision: 0,
        issuer: "".to_string(),
        status: AuditEventSnapshotStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_audit_event_snapshot("code-420", "label-433");
        assert_eq!(item.status, AuditEventSnapshotStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_audit_event_snapshot("code-420", "label-433");
        if item.move_to(AuditEventSnapshotStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventSnapshotStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_audit_event_snapshot("code-420", "label-433");
        item.status = AuditEventSnapshotStatus::Withdrawn;
        assert!(
            item.move_to(AuditEventSnapshotStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
