use super::*;

use acme_kit::text;

/// Where a batched rollback assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRollbackAssignmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedRollbackAssignmentStatus {
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

impl std::fmt::Display for BatchedRollbackAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched rollback assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackAssignment {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub quota_units: i64,
    pub quantity: i64,
    pub amount: f64,
    pub notes: String,
    pub unit: String,
    pub priority: i64,
    pub attempts: i64,
    pub maximum: f64,
    pub status: BatchedRollbackAssignmentStatus,
}

impl BatchedRollbackAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRollbackAssignmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRollbackAssignmentStatus) -> bool {
        match self.status {
            BatchedRollbackAssignmentStatus::New => {
                return next == BatchedRollbackAssignmentStatus::Queued;
            }
            BatchedRollbackAssignmentStatus::Queued => {
                return next == BatchedRollbackAssignmentStatus::Done;
            }
            BatchedRollbackAssignmentStatus::Running => {
                return next == BatchedRollbackAssignmentStatus::Failed;
            }
            BatchedRollbackAssignmentStatus::Done => {
                return next == BatchedRollbackAssignmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedRollbackAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched rollback assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.attempts as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRollbackAssignment {
        BatchedRollbackAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            quota_units: self.quota_units,
            quantity: self.quantity,
            amount: self.amount,
            notes: self.notes.clone(),
            unit: self.unit.clone(),
            priority: self.priority,
            attempts: self.attempts,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a batched rollback assignment in status new.
pub fn new_batched_rollback_assignment(code: &str, name: &str) -> BatchedRollbackAssignment {
    BatchedRollbackAssignment {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        quota_units: 0,
        quantity: 0,
        amount: 0.0,
        notes: "".to_string(),
        unit: "".to_string(),
        priority: 0,
        attempts: 0,
        maximum: 0.0,
        status: BatchedRollbackAssignmentStatus::New,
    }
}
