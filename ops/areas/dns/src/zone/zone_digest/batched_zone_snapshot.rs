use super::*;

use acme_kit::text;

/// Where a batched zone snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedZoneSnapshotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedZoneSnapshotStatus {
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

impl std::fmt::Display for BatchedZoneSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched zone snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedZoneSnapshot {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub attempts: i64,
    pub updated_at: i64,
    pub pinned: bool,
    pub fingerprint: String,
    pub status: BatchedZoneSnapshotStatus,
}

impl BatchedZoneSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedZoneSnapshotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedZoneSnapshotStatus) -> bool {
        match self.status {
            BatchedZoneSnapshotStatus::Open => {
                return next == BatchedZoneSnapshotStatus::Reopened || next == BatchedZoneSnapshotStatus::Archived;
            }
            BatchedZoneSnapshotStatus::InReview => {
                return next == BatchedZoneSnapshotStatus::Resolved || next == BatchedZoneSnapshotStatus::Reopened;
            }
            BatchedZoneSnapshotStatus::Resolved => {
                match next {
                    BatchedZoneSnapshotStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedZoneSnapshotStatus::Reopened => {
                match next {
                    BatchedZoneSnapshotStatus::Archived => {
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

    pub fn move_to(&mut self, next: BatchedZoneSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched zone snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.threshold + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedZoneSnapshot {
        BatchedZoneSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            attempts: self.attempts,
            updated_at: self.updated_at,
            pinned: self.pinned,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched zone snapshot in status open.
pub fn new_batched_zone_snapshot(code: &str, label: &str) -> BatchedZoneSnapshot {
    BatchedZoneSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        attempts: 0,
        updated_at: 0,
        pinned: false,
        fingerprint: "".to_string(),
        status: BatchedZoneSnapshotStatus::Open,
    }
}
