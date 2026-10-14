use super::*;

use acme_kit::text;

/// Where a retention checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionCheckpointStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RetentionCheckpointStatus {
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

impl std::fmt::Display for RetentionCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionCheckpoint {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub threshold: f64,
    pub checksum: String,
    pub weight: f64,
    pub capacity: i64,
    pub source: String,
    pub status: RetentionCheckpointStatus,
}

impl RetentionCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionCheckpointStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionCheckpointStatus) -> bool {
        match self.status {
            RetentionCheckpointStatus::Planned => {
                return next == RetentionCheckpointStatus::Cancelled;
            }
            RetentionCheckpointStatus::Scheduled => {
                return next == RetentionCheckpointStatus::Completed || next == RetentionCheckpointStatus::Cancelled;
            }
            RetentionCheckpointStatus::InProgress => {
                match next {
                    RetentionCheckpointStatus::Completed | RetentionCheckpointStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionCheckpointStatus::Completed => {
                return next == RetentionCheckpointStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.capacity as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionCheckpoint {
        RetentionCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            threshold: self.threshold,
            checksum: self.checksum.clone(),
            weight: self.weight,
            capacity: self.capacity,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a retention checkpoint in status planned.
pub fn new_retention_checkpoint(code: &str, title: &str) -> RetentionCheckpoint {
    RetentionCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        threshold: 0.0,
        checksum: "".to_string(),
        weight: 0.0,
        capacity: 0,
        source: "".to_string(),
        status: RetentionCheckpointStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_retention_checkpoint("code-811", "title-664");
        assert_eq!(item.status, RetentionCheckpointStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_retention_checkpoint("code-811", "title-664");
        if item.move_to(RetentionCheckpointStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionCheckpointStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_retention_checkpoint("code-811", "title-664");
        item.status = RetentionCheckpointStatus::Cancelled;
        assert!(
            item.move_to(RetentionCheckpointStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
