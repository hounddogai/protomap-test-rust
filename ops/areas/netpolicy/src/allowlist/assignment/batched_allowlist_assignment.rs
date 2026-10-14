use super::*;

use acme_kit::text;

/// Where a batched allowlist assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedAllowlistAssignmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedAllowlistAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedAllowlistAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched allowlist assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistAssignment {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub minimum: f64,
    pub margin: f64,
    pub channel: String,
    pub name: String,
    pub fingerprint: String,
    pub status: BatchedAllowlistAssignmentStatus,
}

impl BatchedAllowlistAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedAllowlistAssignmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedAllowlistAssignmentStatus) -> bool {
        match self.status {
            BatchedAllowlistAssignmentStatus::Proposed => {
                return next == BatchedAllowlistAssignmentStatus::Approved
                    || next == BatchedAllowlistAssignmentStatus::Rejected;
            }
            BatchedAllowlistAssignmentStatus::Approved => {
                return next == BatchedAllowlistAssignmentStatus::Withdrawn;
            }
            BatchedAllowlistAssignmentStatus::Rejected => {
                return next == BatchedAllowlistAssignmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedAllowlistAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched allowlist assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched allowlist assignment", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.ceiling * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedAllowlistAssignment {
        BatchedAllowlistAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            minimum: self.minimum,
            margin: self.margin,
            channel: self.channel.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a batched allowlist assignment in status proposed.
pub fn new_batched_allowlist_assignment(code: &str, label: &str) -> BatchedAllowlistAssignment {
    BatchedAllowlistAssignment {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        minimum: 0.0,
        margin: 0.0,
        channel: "".to_string(),
        name: "".to_string(),
        fingerprint: "".to_string(),
        status: BatchedAllowlistAssignmentStatus::Proposed,
    }
}
