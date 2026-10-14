use super::*;

use acme_kit::text;

/// Where a batched grant snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedGrantSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedGrantSnapshotStatus {
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

impl std::fmt::Display for BatchedGrantSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched grant snapshot as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantSnapshot {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub notes: String,
    pub country: String,
    pub attempts: i64,
    pub owner_id: String,
    pub version: i64,
    pub ratio: f64,
    pub burst: i64,
    pub ends_at: i64,
    pub status: BatchedGrantSnapshotStatus,
}

impl BatchedGrantSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedGrantSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedGrantSnapshotStatus) -> bool {
        match self.status {
            BatchedGrantSnapshotStatus::New => {
                match next {
                    BatchedGrantSnapshotStatus::Running | BatchedGrantSnapshotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedGrantSnapshotStatus::Queued => {
                match next {
                    BatchedGrantSnapshotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedGrantSnapshotStatus::Running => {
                return next == BatchedGrantSnapshotStatus::Done || next == BatchedGrantSnapshotStatus::Failed;
            }
            BatchedGrantSnapshotStatus::Done => {
                return next == BatchedGrantSnapshotStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedGrantSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched grant snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.version as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedGrantSnapshot {
        BatchedGrantSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            notes: self.notes.clone(),
            country: self.country.clone(),
            attempts: self.attempts,
            owner_id: self.owner_id.clone(),
            version: self.version,
            ratio: self.ratio,
            burst: self.burst,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a batched grant snapshot in status new.
pub fn new_batched_grant_snapshot(code: &str, label: &str) -> BatchedGrantSnapshot {
    BatchedGrantSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        notes: "".to_string(),
        country: "".to_string(),
        attempts: 0,
        owner_id: "".to_string(),
        version: 0,
        ratio: 0.0,
        burst: 0,
        ends_at: 0,
        status: BatchedGrantSnapshotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_batched_grant_snapshot("code-239", "label-50");
        assert_eq!(item.status, BatchedGrantSnapshotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_batched_grant_snapshot("code-239", "label-50");
        if item.move_to(BatchedGrantSnapshotStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedGrantSnapshotStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_batched_grant_snapshot("code-239", "label-50");
        item.status = BatchedGrantSnapshotStatus::Failed;
        assert!(
            item.move_to(BatchedGrantSnapshotStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
