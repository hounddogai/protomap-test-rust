use super::*;

use acme_kit::text;

/// Where a batched rollback batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRollbackBatchStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedRollbackBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedRollbackBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched rollback batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackBatch {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub unit: String,
    pub budget: f64,
    pub expires_at: i64,
    pub rotation_days: i64,
    pub title: String,
    pub rate: f64,
    pub duration_minutes: i64,
    pub created_at: i64,
    pub status: BatchedRollbackBatchStatus,
}

impl BatchedRollbackBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRollbackBatchStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRollbackBatchStatus) -> bool {
        match self.status {
            BatchedRollbackBatchStatus::Proposed => {
                match next {
                    BatchedRollbackBatchStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRollbackBatchStatus::Approved => {
                match next {
                    BatchedRollbackBatchStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRollbackBatchStatus::Rejected => {
                match next {
                    BatchedRollbackBatchStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: BatchedRollbackBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched rollback batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.duration_minutes as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRollbackBatch {
        BatchedRollbackBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            unit: self.unit.clone(),
            budget: self.budget,
            expires_at: self.expires_at,
            rotation_days: self.rotation_days,
            title: self.title.clone(),
            rate: self.rate,
            duration_minutes: self.duration_minutes,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a batched rollback batch in status proposed.
pub fn new_batched_rollback_batch(code: &str, label: &str) -> BatchedRollbackBatch {
    BatchedRollbackBatch {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        unit: "".to_string(),
        budget: 0.0,
        expires_at: 0,
        rotation_days: 0,
        title: "".to_string(),
        rate: 0.0,
        duration_minutes: 0,
        created_at: 0,
        status: BatchedRollbackBatchStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_rollback_batch("code-500", "label-79");
        assert_eq!(item.status, BatchedRollbackBatchStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_batched_rollback_batch("code-500", "label-79");
        if item.move_to(BatchedRollbackBatchStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRollbackBatchStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_rollback_batch("code-500", "label-79");
        item.status = BatchedRollbackBatchStatus::Withdrawn;
        assert!(
            item.move_to(BatchedRollbackBatchStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
