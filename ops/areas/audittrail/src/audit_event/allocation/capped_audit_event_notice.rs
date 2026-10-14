use super::*;

use acme_kit::text;

/// Where a capped audit event notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedAuditEventNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedAuditEventNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedAuditEventNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped audit event notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedAuditEventNotice {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub amount: f64,
    pub reviewer: String,
    pub audience: String,
    pub status: CappedAuditEventNoticeStatus,
}

impl CappedAuditEventNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedAuditEventNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedAuditEventNoticeStatus) -> bool {
        match self.status {
            CappedAuditEventNoticeStatus::Proposed => {
                return next == CappedAuditEventNoticeStatus::Rejected;
            }
            CappedAuditEventNoticeStatus::Approved => {
                return next == CappedAuditEventNoticeStatus::Rejected
                    || next == CappedAuditEventNoticeStatus::Withdrawn;
            }
            CappedAuditEventNoticeStatus::Rejected => {
                return next == CappedAuditEventNoticeStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedAuditEventNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped audit event notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.duration_minutes as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedAuditEventNotice {
        CappedAuditEventNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            amount: self.amount,
            reviewer: self.reviewer.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped audit event notice in status proposed.
pub fn new_capped_audit_event_notice(code: &str, label: &str) -> CappedAuditEventNotice {
    CappedAuditEventNotice {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        amount: 0.0,
        reviewer: "".to_string(),
        audience: "".to_string(),
        status: CappedAuditEventNoticeStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_capped_audit_event_notice("code-514", "label-53");
        assert_eq!(item.status, CappedAuditEventNoticeStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_capped_audit_event_notice("code-514", "label-53");
        if item.move_to(CappedAuditEventNoticeStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedAuditEventNoticeStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_capped_audit_event_notice("code-514", "label-53");
        item.status = CappedAuditEventNoticeStatus::Withdrawn;
        assert!(
            item.move_to(CappedAuditEventNoticeStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
