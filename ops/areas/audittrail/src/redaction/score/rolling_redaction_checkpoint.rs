use super::*;

use acme_kit::text;

/// Where a rolling redaction checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingRedactionCheckpointStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingRedactionCheckpointStatus {
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

impl std::fmt::Display for RollingRedactionCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling redaction checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionCheckpoint {
    pub code: String,
    pub title: String,
    pub description: String,
    pub rate: f64,
    pub fingerprint: String,
    pub segment: String,
    pub floor_value: f64,
    pub attempts: i64,
    pub retries: i64,
    pub status: RollingRedactionCheckpointStatus,
}

impl RollingRedactionCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingRedactionCheckpointStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingRedactionCheckpointStatus) -> bool {
        match self.status {
            RollingRedactionCheckpointStatus::New => {
                return next == RollingRedactionCheckpointStatus::Running;
            }
            RollingRedactionCheckpointStatus::Queued => {
                match next {
                    RollingRedactionCheckpointStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRedactionCheckpointStatus::Running => {
                match next {
                    RollingRedactionCheckpointStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRedactionCheckpointStatus::Done => {
                return next == RollingRedactionCheckpointStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingRedactionCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling redaction checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.attempts as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingRedactionCheckpoint {
        RollingRedactionCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            rate: self.rate,
            fingerprint: self.fingerprint.clone(),
            segment: self.segment.clone(),
            floor_value: self.floor_value,
            attempts: self.attempts,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rolling redaction checkpoint in status new.
pub fn new_rolling_redaction_checkpoint(code: &str, title: &str) -> RollingRedactionCheckpoint {
    RollingRedactionCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        description: "".to_string(),
        rate: 0.0,
        fingerprint: "".to_string(),
        segment: "".to_string(),
        floor_value: 0.0,
        attempts: 0,
        retries: 0,
        status: RollingRedactionCheckpointStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_redaction_checkpoint("code-745", "title-13");
        assert_eq!(item.status, RollingRedactionCheckpointStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_rolling_redaction_checkpoint("code-745", "title-13");
        if item.move_to(RollingRedactionCheckpointStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingRedactionCheckpointStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_redaction_checkpoint("code-745", "title-13");
        item.status = RollingRedactionCheckpointStatus::Failed;
        assert!(
            item.move_to(RollingRedactionCheckpointStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
