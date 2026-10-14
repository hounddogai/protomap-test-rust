use super::*;

use acme_kit::text;

/// Where a batched isolation record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedIsolationRecordStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedIsolationRecordStatus {
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

impl std::fmt::Display for BatchedIsolationRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched isolation record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIsolationRecord {
    pub code: String,
    pub name: String,
    pub region: String,
    pub segment: String,
    pub attempts: i64,
    pub scope: String,
    pub active: bool,
    pub status: BatchedIsolationRecordStatus,
}

impl BatchedIsolationRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedIsolationRecordStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedIsolationRecordStatus) -> bool {
        match self.status {
            BatchedIsolationRecordStatus::New => {
                match next {
                    BatchedIsolationRecordStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedIsolationRecordStatus::Queued => {
                return next == BatchedIsolationRecordStatus::Running;
            }
            BatchedIsolationRecordStatus::Running => {
                match next {
                    BatchedIsolationRecordStatus::Done | BatchedIsolationRecordStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedIsolationRecordStatus::Done => {
                match next {
                    BatchedIsolationRecordStatus::Failed => {
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

    pub fn move_to(&mut self, next: BatchedIsolationRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched isolation record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.attempts as f64
    }

    pub fn copy(&self) -> BatchedIsolationRecord {
        BatchedIsolationRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            segment: self.segment.clone(),
            attempts: self.attempts,
            scope: self.scope.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a batched isolation record in status new.
pub fn new_batched_isolation_record(code: &str, name: &str) -> BatchedIsolationRecord {
    BatchedIsolationRecord {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        segment: "".to_string(),
        attempts: 0,
        scope: "".to_string(),
        active: false,
        status: BatchedIsolationRecordStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_batched_isolation_record("code-28", "name-917");
        assert_eq!(item.status, BatchedIsolationRecordStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_batched_isolation_record("code-28", "name-917");
        if item.move_to(BatchedIsolationRecordStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedIsolationRecordStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_batched_isolation_record("code-28", "name-917");
        item.status = BatchedIsolationRecordStatus::Failed;
        assert!(
            item.move_to(BatchedIsolationRecordStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
