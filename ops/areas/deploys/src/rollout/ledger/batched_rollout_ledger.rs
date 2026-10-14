use super::*;

use acme_kit::text;

/// Where a batched rollout ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRolloutLedgerStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedRolloutLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedRolloutLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched rollout ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRolloutLedger {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub active: bool,
    pub visible: bool,
    pub starts_at: i64,
    pub burst: i64,
    pub floor_value: f64,
    pub status: BatchedRolloutLedgerStatus,
}

impl BatchedRolloutLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRolloutLedgerStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRolloutLedgerStatus) -> bool {
        match self.status {
            BatchedRolloutLedgerStatus::Proposed => {
                return next == BatchedRolloutLedgerStatus::Withdrawn;
            }
            BatchedRolloutLedgerStatus::Approved => {
                return next == BatchedRolloutLedgerStatus::Rejected || next == BatchedRolloutLedgerStatus::Withdrawn;
            }
            BatchedRolloutLedgerStatus::Rejected => {
                match next {
                    BatchedRolloutLedgerStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: BatchedRolloutLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched rollout ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.burst as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRolloutLedger {
        BatchedRolloutLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            active: self.active,
            visible: self.visible,
            starts_at: self.starts_at,
            burst: self.burst,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a batched rollout ledger in status proposed.
pub fn new_batched_rollout_ledger(code: &str, name: &str) -> BatchedRolloutLedger {
    BatchedRolloutLedger {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        active: false,
        visible: false,
        starts_at: 0,
        burst: 0,
        floor_value: 0.0,
        status: BatchedRolloutLedgerStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_rollout_ledger("code-931", "name-284");
        assert_eq!(item.status, BatchedRolloutLedgerStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_batched_rollout_ledger("code-931", "name-284");
        if item.move_to(BatchedRolloutLedgerStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRolloutLedgerStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_rollout_ledger("code-931", "name-284");
        item.status = BatchedRolloutLedgerStatus::Withdrawn;
        assert!(
            item.move_to(BatchedRolloutLedgerStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
