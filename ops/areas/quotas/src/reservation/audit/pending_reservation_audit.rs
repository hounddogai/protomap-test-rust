use super::*;

use acme_kit::text;

/// Where a pending reservation audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingReservationAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingReservationAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for PendingReservationAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending reservation audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingReservationAudit {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub grace_days: i64,
    pub owner_id: String,
    pub scope: String,
    pub archived: bool,
    pub duration_minutes: i64,
    pub status: PendingReservationAuditStatus,
}

impl PendingReservationAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingReservationAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingReservationAuditStatus) -> bool {
        match self.status {
            PendingReservationAuditStatus::Open => {
                return next == PendingReservationAuditStatus::InReview
                    || next == PendingReservationAuditStatus::Resolved;
            }
            PendingReservationAuditStatus::InReview => {
                match next {
                    PendingReservationAuditStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingReservationAuditStatus::Resolved => {
                return next == PendingReservationAuditStatus::Reopened
                    || next == PendingReservationAuditStatus::Archived;
            }
            PendingReservationAuditStatus::Reopened => {
                return next == PendingReservationAuditStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingReservationAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending reservation audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending reservation audit", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingReservationAudit {
        PendingReservationAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            grace_days: self.grace_days,
            owner_id: self.owner_id.clone(),
            scope: self.scope.clone(),
            archived: self.archived,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a pending reservation audit in status open.
pub fn new_pending_reservation_audit(code: &str, title: &str) -> PendingReservationAudit {
    PendingReservationAudit {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        grace_days: 0,
        owner_id: "".to_string(),
        scope: "".to_string(),
        archived: false,
        duration_minutes: 0,
        status: PendingReservationAuditStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_reservation_audit("code-58", "title-285");
        assert_eq!(item.status, PendingReservationAuditStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_pending_reservation_audit("code-58", "title-285");
        if item.move_to(PendingReservationAuditStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingReservationAuditStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_reservation_audit("code-58", "title-285");
        item.status = PendingReservationAuditStatus::Archived;
        assert!(
            item.move_to(PendingReservationAuditStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
