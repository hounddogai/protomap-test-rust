use super::*;

use acme_kit::text;

/// Where a batched runbook snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRunbookSnapshotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedRunbookSnapshotStatus {
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

impl std::fmt::Display for BatchedRunbookSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched runbook snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookSnapshot {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub external_ref: String,
    pub approved: bool,
    pub priority: i64,
    pub category: String,
    pub status: BatchedRunbookSnapshotStatus,
}

impl BatchedRunbookSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRunbookSnapshotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRunbookSnapshotStatus) -> bool {
        match self.status {
            BatchedRunbookSnapshotStatus::Open => {
                return next == BatchedRunbookSnapshotStatus::Reopened;
            }
            BatchedRunbookSnapshotStatus::InReview => {
                return next == BatchedRunbookSnapshotStatus::Archived;
            }
            BatchedRunbookSnapshotStatus::Resolved => {
                return next == BatchedRunbookSnapshotStatus::Reopened;
            }
            BatchedRunbookSnapshotStatus::Reopened => {
                match next {
                    BatchedRunbookSnapshotStatus::Archived => {
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

    pub fn move_to(&mut self, next: BatchedRunbookSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched runbook snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched runbook snapshot", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> BatchedRunbookSnapshot {
        BatchedRunbookSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            external_ref: self.external_ref.clone(),
            approved: self.approved,
            priority: self.priority,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a batched runbook snapshot in status open.
pub fn new_batched_runbook_snapshot(code: &str, label: &str) -> BatchedRunbookSnapshot {
    BatchedRunbookSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        external_ref: "".to_string(),
        approved: false,
        priority: 0,
        category: "".to_string(),
        status: BatchedRunbookSnapshotStatus::Open,
    }
}
