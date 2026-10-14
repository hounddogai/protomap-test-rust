use super::*;

use acme_kit::text;

/// Where a approval snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalSnapshotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ApprovalSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ApprovalSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSnapshot {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub phone: String,
    pub source: String,
    pub key_id: String,
    pub retries: i64,
    pub threshold: f64,
    pub notes: String,
    pub status: ApprovalSnapshotStatus,
}

impl ApprovalSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalSnapshotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalSnapshotStatus) -> bool {
        match self.status {
            ApprovalSnapshotStatus::Draft => {
                return next == ApprovalSnapshotStatus::Pending;
            }
            ApprovalSnapshotStatus::Pending => {
                match next {
                    ApprovalSnapshotStatus::Active | ApprovalSnapshotStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalSnapshotStatus::Active => {
                match next {
                    ApprovalSnapshotStatus::Suspended | ApprovalSnapshotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalSnapshotStatus::Suspended => {
                return next == ApprovalSnapshotStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.threshold + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalSnapshot {
        ApprovalSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            phone: self.phone.clone(),
            source: self.source.clone(),
            key_id: self.key_id.clone(),
            retries: self.retries,
            threshold: self.threshold,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a approval snapshot in status draft.
pub fn new_approval_snapshot(code: &str, label: &str) -> ApprovalSnapshot {
    ApprovalSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        phone: "".to_string(),
        source: "".to_string(),
        key_id: "".to_string(),
        retries: 0,
        threshold: 0.0,
        notes: "".to_string(),
        status: ApprovalSnapshotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_approval_snapshot("code-786", "label-333");
        assert_eq!(item.status, ApprovalSnapshotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_approval_snapshot("code-786", "label-333");
        if item.move_to(ApprovalSnapshotStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalSnapshotStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_approval_snapshot("code-786", "label-333");
        item.status = ApprovalSnapshotStatus::Closed;
        assert!(
            item.move_to(ApprovalSnapshotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
