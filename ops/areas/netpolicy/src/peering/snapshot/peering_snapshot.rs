use super::*;

use acme_kit::text;

/// Where a peering snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PeeringSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PeeringSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSnapshot {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub attempts: i64,
    pub maximum: f64,
    pub status: PeeringSnapshotStatus,
}

impl PeeringSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringSnapshotStatus) -> bool {
        match self.status {
            PeeringSnapshotStatus::New => {
                return next == PeeringSnapshotStatus::Queued || next == PeeringSnapshotStatus::Done;
            }
            PeeringSnapshotStatus::Queued => {
                match next {
                    PeeringSnapshotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringSnapshotStatus::Running => {
                match next {
                    PeeringSnapshotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringSnapshotStatus::Done => {
                return next == PeeringSnapshotStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.maximum + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringSnapshot {
        PeeringSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            attempts: self.attempts,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a peering snapshot in status new.
pub fn new_peering_snapshot(code: &str, title: &str) -> PeeringSnapshot {
    PeeringSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        attempts: 0,
        maximum: 0.0,
        status: PeeringSnapshotStatus::New,
    }
}
