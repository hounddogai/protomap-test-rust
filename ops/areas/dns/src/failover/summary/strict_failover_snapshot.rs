use super::*;

use acme_kit::text;

/// Where a strict failover snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictFailoverSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictFailoverSnapshotStatus {
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

impl std::fmt::Display for StrictFailoverSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict failover snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFailoverSnapshot {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub rotation_days: i64,
    pub audience: String,
    pub status: StrictFailoverSnapshotStatus,
}

impl StrictFailoverSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictFailoverSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictFailoverSnapshotStatus) -> bool {
        match self.status {
            StrictFailoverSnapshotStatus::New => {
                match next {
                    StrictFailoverSnapshotStatus::Queued | StrictFailoverSnapshotStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictFailoverSnapshotStatus::Queued => {
                return next == StrictFailoverSnapshotStatus::Running || next == StrictFailoverSnapshotStatus::Done;
            }
            StrictFailoverSnapshotStatus::Running => {
                match next {
                    StrictFailoverSnapshotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictFailoverSnapshotStatus::Done => {
                return next == StrictFailoverSnapshotStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictFailoverSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict failover snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict failover snapshot", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.rotation_days as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictFailoverSnapshot {
        StrictFailoverSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            rotation_days: self.rotation_days,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict failover snapshot in status new.
pub fn new_strict_failover_snapshot(code: &str, name: &str) -> StrictFailoverSnapshot {
    StrictFailoverSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        rotation_days: 0,
        audience: "".to_string(),
        status: StrictFailoverSnapshotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_strict_failover_snapshot("code-867", "name-909");
        assert_eq!(item.status, StrictFailoverSnapshotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_strict_failover_snapshot("code-867", "name-909");
        if item.move_to(StrictFailoverSnapshotStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictFailoverSnapshotStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_strict_failover_snapshot("code-867", "name-909");
        item.status = StrictFailoverSnapshotStatus::Failed;
        assert!(
            item.move_to(StrictFailoverSnapshotStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
