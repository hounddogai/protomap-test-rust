use super::*;

use acme_kit::text;

/// Where a limit checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitCheckpointStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LimitCheckpointStatus {
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

impl std::fmt::Display for LimitCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitCheckpoint {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub summary: String,
    pub retries: i64,
    pub rotation_days: i64,
    pub discount: f64,
    pub status: LimitCheckpointStatus,
}

impl LimitCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitCheckpointStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitCheckpointStatus) -> bool {
        match self.status {
            LimitCheckpointStatus::New => {
                match next {
                    LimitCheckpointStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitCheckpointStatus::Queued => {
                match next {
                    LimitCheckpointStatus::Done | LimitCheckpointStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitCheckpointStatus::Running => {
                return next == LimitCheckpointStatus::Failed;
            }
            LimitCheckpointStatus::Done => {
                return next == LimitCheckpointStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.retries as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitCheckpoint {
        LimitCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            summary: self.summary.clone(),
            retries: self.retries,
            rotation_days: self.rotation_days,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a limit checkpoint in status new.
pub fn new_limit_checkpoint(code: &str, title: &str) -> LimitCheckpoint {
    LimitCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        summary: "".to_string(),
        retries: 0,
        rotation_days: 0,
        discount: 0.0,
        status: LimitCheckpointStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_limit_checkpoint("code-993", "title-940");
        assert_eq!(item.status, LimitCheckpointStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_limit_checkpoint("code-993", "title-940");
        if item.move_to(LimitCheckpointStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitCheckpointStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_limit_checkpoint("code-993", "title-940");
        item.status = LimitCheckpointStatus::Failed;
        assert!(
            item.move_to(LimitCheckpointStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
