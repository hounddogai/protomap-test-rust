use super::*;

use acme_kit::text;

/// Where a batched slo assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedSloAssignmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedSloAssignmentStatus {
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

impl std::fmt::Display for BatchedSloAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched slo assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSloAssignment {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub country: String,
    pub notes: String,
    pub quantity: i64,
    pub status: BatchedSloAssignmentStatus,
}

impl BatchedSloAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedSloAssignmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedSloAssignmentStatus) -> bool {
        match self.status {
            BatchedSloAssignmentStatus::New => {
                return next == BatchedSloAssignmentStatus::Running || next == BatchedSloAssignmentStatus::Failed;
            }
            BatchedSloAssignmentStatus::Queued => {
                return next == BatchedSloAssignmentStatus::Running || next == BatchedSloAssignmentStatus::Done;
            }
            BatchedSloAssignmentStatus::Running => {
                return next == BatchedSloAssignmentStatus::Done || next == BatchedSloAssignmentStatus::Failed;
            }
            BatchedSloAssignmentStatus::Done => {
                return next == BatchedSloAssignmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedSloAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched slo assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> BatchedSloAssignment {
        BatchedSloAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            country: self.country.clone(),
            notes: self.notes.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a batched slo assignment in status new.
pub fn new_batched_slo_assignment(code: &str, title: &str) -> BatchedSloAssignment {
    BatchedSloAssignment {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        country: "".to_string(),
        notes: "".to_string(),
        quantity: 0,
        status: BatchedSloAssignmentStatus::New,
    }
}
