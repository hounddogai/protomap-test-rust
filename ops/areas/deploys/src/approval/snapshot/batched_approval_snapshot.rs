use super::*;

use acme_kit::text;

/// Where a batched approval snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedApprovalSnapshotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedApprovalSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for BatchedApprovalSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched approval snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedApprovalSnapshot {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub batch_size: i64,
    pub phone: String,
    pub archived: bool,
    pub budget: f64,
    pub status: BatchedApprovalSnapshotStatus,
}

impl BatchedApprovalSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedApprovalSnapshotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedApprovalSnapshotStatus) -> bool {
        match self.status {
            BatchedApprovalSnapshotStatus::Idle => {
                return next == BatchedApprovalSnapshotStatus::Ready;
            }
            BatchedApprovalSnapshotStatus::Warming => {
                match next {
                    BatchedApprovalSnapshotStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedApprovalSnapshotStatus::Ready => {
                match next {
                    BatchedApprovalSnapshotStatus::Draining | BatchedApprovalSnapshotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedApprovalSnapshotStatus::Draining => {
                match next {
                    BatchedApprovalSnapshotStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BatchedApprovalSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched approval snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.batch_size as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedApprovalSnapshot {
        BatchedApprovalSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            batch_size: self.batch_size,
            phone: self.phone.clone(),
            archived: self.archived,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched approval snapshot in status idle.
pub fn new_batched_approval_snapshot(code: &str, name: &str) -> BatchedApprovalSnapshot {
    BatchedApprovalSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        batch_size: 0,
        phone: "".to_string(),
        archived: false,
        budget: 0.0,
        status: BatchedApprovalSnapshotStatus::Idle,
    }
}
