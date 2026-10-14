use super::*;

use acme_kit::text;

/// Where a suspension checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionCheckpointStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SuspensionCheckpointStatus {
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

impl std::fmt::Display for SuspensionCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a suspension checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionCheckpoint {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub fingerprint: String,
    pub quota_units: i64,
    pub revision: i64,
    pub owner_id: String,
    pub budget: f64,
    pub status: SuspensionCheckpointStatus,
}

impl SuspensionCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionCheckpointStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionCheckpointStatus) -> bool {
        match self.status {
            SuspensionCheckpointStatus::New => {
                match next {
                    SuspensionCheckpointStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionCheckpointStatus::Queued => {
                return next == SuspensionCheckpointStatus::Running || next == SuspensionCheckpointStatus::Failed;
            }
            SuspensionCheckpointStatus::Running => {
                return next == SuspensionCheckpointStatus::Done;
            }
            SuspensionCheckpointStatus::Done => {
                match next {
                    SuspensionCheckpointStatus::Failed => {
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

    pub fn move_to(&mut self, next: SuspensionCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "suspension checkpoint", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.revision as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionCheckpoint {
        SuspensionCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            fingerprint: self.fingerprint.clone(),
            quota_units: self.quota_units,
            revision: self.revision,
            owner_id: self.owner_id.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a suspension checkpoint in status new.
pub fn new_suspension_checkpoint(code: &str, name: &str) -> SuspensionCheckpoint {
    SuspensionCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        fingerprint: "".to_string(),
        quota_units: 0,
        revision: 0,
        owner_id: "".to_string(),
        budget: 0.0,
        status: SuspensionCheckpointStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_suspension_checkpoint("code-456", "name-283");
        assert_eq!(item.status, SuspensionCheckpointStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_suspension_checkpoint("code-456", "name-283");
        if item.move_to(SuspensionCheckpointStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionCheckpointStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_suspension_checkpoint("code-456", "name-283");
        item.status = SuspensionCheckpointStatus::Failed;
        assert!(
            item.move_to(SuspensionCheckpointStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
