use super::*;

use acme_kit::text;

/// Where a batched delegation checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedDelegationCheckpointStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedDelegationCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedDelegationCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched delegation checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDelegationCheckpoint {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub approved: bool,
    pub batch_size: i64,
    pub threshold: f64,
    pub key_id: String,
    pub status: BatchedDelegationCheckpointStatus,
}

impl BatchedDelegationCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedDelegationCheckpointStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedDelegationCheckpointStatus) -> bool {
        match self.status {
            BatchedDelegationCheckpointStatus::Proposed => {
                match next {
                    BatchedDelegationCheckpointStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedDelegationCheckpointStatus::Approved => {
                return next == BatchedDelegationCheckpointStatus::Withdrawn;
            }
            BatchedDelegationCheckpointStatus::Rejected => {
                return next == BatchedDelegationCheckpointStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedDelegationCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched delegation checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.batch_size as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedDelegationCheckpoint {
        BatchedDelegationCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            approved: self.approved,
            batch_size: self.batch_size,
            threshold: self.threshold,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched delegation checkpoint in status proposed.
pub fn new_batched_delegation_checkpoint(code: &str, label: &str) -> BatchedDelegationCheckpoint {
    BatchedDelegationCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        approved: false,
        batch_size: 0,
        threshold: 0.0,
        key_id: "".to_string(),
        status: BatchedDelegationCheckpointStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_delegation_checkpoint("code-397", "label-313");
        assert_eq!(item.status, BatchedDelegationCheckpointStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_batched_delegation_checkpoint("code-397", "label-313");
        if item.move_to(BatchedDelegationCheckpointStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedDelegationCheckpointStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_delegation_checkpoint("code-397", "label-313");
        item.status = BatchedDelegationCheckpointStatus::Withdrawn;
        assert!(
            item.move_to(BatchedDelegationCheckpointStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
