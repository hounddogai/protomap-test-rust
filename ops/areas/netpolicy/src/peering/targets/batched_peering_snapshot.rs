use super::*;

use acme_kit::text;

/// Where a batched peering snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPeeringSnapshotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedPeeringSnapshotStatus {
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

impl std::fmt::Display for BatchedPeeringSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched peering snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPeeringSnapshot {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub summary: String,
    pub audience: String,
    pub currency: String,
    pub batch_size: i64,
    pub principal: String,
    pub status: BatchedPeeringSnapshotStatus,
}

impl BatchedPeeringSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPeeringSnapshotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPeeringSnapshotStatus) -> bool {
        match self.status {
            BatchedPeeringSnapshotStatus::Planned => {
                match next {
                    BatchedPeeringSnapshotStatus::InProgress | BatchedPeeringSnapshotStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPeeringSnapshotStatus::Scheduled => {
                match next {
                    BatchedPeeringSnapshotStatus::Completed | BatchedPeeringSnapshotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPeeringSnapshotStatus::InProgress => {
                match next {
                    BatchedPeeringSnapshotStatus::Completed | BatchedPeeringSnapshotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPeeringSnapshotStatus::Completed => {
                return next == BatchedPeeringSnapshotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedPeeringSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched peering snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched peering snapshot", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> BatchedPeeringSnapshot {
        BatchedPeeringSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            summary: self.summary.clone(),
            audience: self.audience.clone(),
            currency: self.currency.clone(),
            batch_size: self.batch_size,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched peering snapshot in status planned.
pub fn new_batched_peering_snapshot(code: &str, name: &str) -> BatchedPeeringSnapshot {
    BatchedPeeringSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        summary: "".to_string(),
        audience: "".to_string(),
        currency: "".to_string(),
        batch_size: 0,
        principal: "".to_string(),
        status: BatchedPeeringSnapshotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_batched_peering_snapshot("code-47", "name-30");
        assert_eq!(item.status, BatchedPeeringSnapshotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_batched_peering_snapshot("code-47", "name-30");
        if item.move_to(BatchedPeeringSnapshotStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedPeeringSnapshotStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_batched_peering_snapshot("code-47", "name-30");
        item.status = BatchedPeeringSnapshotStatus::Cancelled;
        assert!(
            item.move_to(BatchedPeeringSnapshotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
