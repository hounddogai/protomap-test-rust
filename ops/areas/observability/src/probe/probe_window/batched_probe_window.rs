use super::*;

use acme_kit::text;

/// Where a batched probe window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedProbeWindowStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedProbeWindowStatus {
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

impl std::fmt::Display for BatchedProbeWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched probe window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeWindow {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub port: i64,
    pub notes: String,
    pub checksum: String,
    pub pinned: bool,
    pub status: BatchedProbeWindowStatus,
}

impl BatchedProbeWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedProbeWindowStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedProbeWindowStatus) -> bool {
        match self.status {
            BatchedProbeWindowStatus::Planned => {
                return next == BatchedProbeWindowStatus::Completed;
            }
            BatchedProbeWindowStatus::Scheduled => {
                match next {
                    BatchedProbeWindowStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedProbeWindowStatus::InProgress => {
                return next == BatchedProbeWindowStatus::Completed || next == BatchedProbeWindowStatus::Cancelled;
            }
            BatchedProbeWindowStatus::Completed => {
                match next {
                    BatchedProbeWindowStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: BatchedProbeWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched probe window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched probe window", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.revision as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedProbeWindow {
        BatchedProbeWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            port: self.port,
            notes: self.notes.clone(),
            checksum: self.checksum.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched probe window in status planned.
pub fn new_batched_probe_window(code: &str, name: &str) -> BatchedProbeWindow {
    BatchedProbeWindow {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        port: 0,
        notes: "".to_string(),
        checksum: "".to_string(),
        pinned: false,
        status: BatchedProbeWindowStatus::Planned,
    }
}
