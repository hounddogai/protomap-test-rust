use super::*;

use acme_kit::text;

/// Where a default segment checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultSegmentCheckpointStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultSegmentCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for DefaultSegmentCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default segment checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSegmentCheckpoint {
    pub code: String,
    pub label: String,
    pub host: String,
    pub rotation_days: i64,
    pub updated_at: i64,
    pub status: DefaultSegmentCheckpointStatus,
}

impl DefaultSegmentCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultSegmentCheckpointStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultSegmentCheckpointStatus) -> bool {
        match self.status {
            DefaultSegmentCheckpointStatus::Planned => {
                match next {
                    DefaultSegmentCheckpointStatus::InProgress | DefaultSegmentCheckpointStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultSegmentCheckpointStatus::Scheduled => {
                return next == DefaultSegmentCheckpointStatus::Cancelled;
            }
            DefaultSegmentCheckpointStatus::InProgress => {
                return next == DefaultSegmentCheckpointStatus::Completed
                    || next == DefaultSegmentCheckpointStatus::Cancelled;
            }
            DefaultSegmentCheckpointStatus::Completed => {
                match next {
                    DefaultSegmentCheckpointStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DefaultSegmentCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default segment checkpoint {} cannot move from {} to {}",
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
        format!("{} {}", "default segment checkpoint", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.rotation_days as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultSegmentCheckpoint {
        DefaultSegmentCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            rotation_days: self.rotation_days,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a default segment checkpoint in status planned.
pub fn new_default_segment_checkpoint(code: &str, label: &str) -> DefaultSegmentCheckpoint {
    DefaultSegmentCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        rotation_days: 0,
        updated_at: 0,
        status: DefaultSegmentCheckpointStatus::Planned,
    }
}
