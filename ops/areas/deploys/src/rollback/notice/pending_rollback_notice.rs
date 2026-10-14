use super::*;

use acme_kit::text;

/// Where a pending rollback notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRollbackNoticeStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingRollbackNoticeStatus {
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

impl std::fmt::Display for PendingRollbackNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending rollback notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRollbackNotice {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub phone: String,
    pub grace_days: i64,
    pub status: PendingRollbackNoticeStatus,
}

impl PendingRollbackNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRollbackNoticeStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRollbackNoticeStatus) -> bool {
        match self.status {
            PendingRollbackNoticeStatus::Open => {
                match next {
                    PendingRollbackNoticeStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRollbackNoticeStatus::InReview => {
                return next == PendingRollbackNoticeStatus::Resolved || next == PendingRollbackNoticeStatus::Archived;
            }
            PendingRollbackNoticeStatus::Resolved => {
                match next {
                    PendingRollbackNoticeStatus::Reopened | PendingRollbackNoticeStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRollbackNoticeStatus::Reopened => {
                match next {
                    PendingRollbackNoticeStatus::Archived => {
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

    pub fn move_to(&mut self, next: PendingRollbackNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending rollback notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> PendingRollbackNotice {
        PendingRollbackNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            phone: self.phone.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending rollback notice in status open.
pub fn new_pending_rollback_notice(code: &str, label: &str) -> PendingRollbackNotice {
    PendingRollbackNotice {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        phone: "".to_string(),
        grace_days: 0,
        status: PendingRollbackNoticeStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_rollback_notice("code-382", "label-440");
        assert_eq!(item.status, PendingRollbackNoticeStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_pending_rollback_notice("code-382", "label-440");
        if item.move_to(PendingRollbackNoticeStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRollbackNoticeStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_rollback_notice("code-382", "label-440");
        item.status = PendingRollbackNoticeStatus::Archived;
        assert!(
            item.move_to(PendingRollbackNoticeStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
