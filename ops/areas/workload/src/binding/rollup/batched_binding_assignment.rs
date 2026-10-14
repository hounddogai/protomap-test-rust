use super::*;

use acme_kit::text;

/// Where a batched binding assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedBindingAssignmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedBindingAssignmentStatus {
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

impl std::fmt::Display for BatchedBindingAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched binding assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingAssignment {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub cluster: String,
    pub minimum: f64,
    pub amount: f64,
    pub quota_units: i64,
    pub attempts: i64,
    pub unit: String,
    pub rate: f64,
    pub status: BatchedBindingAssignmentStatus,
}

impl BatchedBindingAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedBindingAssignmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedBindingAssignmentStatus) -> bool {
        match self.status {
            BatchedBindingAssignmentStatus::New => {
                return next == BatchedBindingAssignmentStatus::Running
                    || next == BatchedBindingAssignmentStatus::Failed;
            }
            BatchedBindingAssignmentStatus::Queued => {
                match next {
                    BatchedBindingAssignmentStatus::Running | BatchedBindingAssignmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedBindingAssignmentStatus::Running => {
                return next == BatchedBindingAssignmentStatus::Done || next == BatchedBindingAssignmentStatus::Failed;
            }
            BatchedBindingAssignmentStatus::Done => {
                match next {
                    BatchedBindingAssignmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: BatchedBindingAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched binding assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched binding assignment", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.rate + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedBindingAssignment {
        BatchedBindingAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            cluster: self.cluster.clone(),
            minimum: self.minimum,
            amount: self.amount,
            quota_units: self.quota_units,
            attempts: self.attempts,
            unit: self.unit.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a batched binding assignment in status new.
pub fn new_batched_binding_assignment(code: &str, name: &str) -> BatchedBindingAssignment {
    BatchedBindingAssignment {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        cluster: "".to_string(),
        minimum: 0.0,
        amount: 0.0,
        quota_units: 0,
        attempts: 0,
        unit: "".to_string(),
        rate: 0.0,
        status: BatchedBindingAssignmentStatus::New,
    }
}
