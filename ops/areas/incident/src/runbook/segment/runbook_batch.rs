use super::*;

use acme_kit::text;

/// Where a runbook batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RunbookBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for RunbookBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook batch as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBatch {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub floor_value: f64,
    pub active: bool,
    pub status: RunbookBatchStatus,
}

impl RunbookBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookBatchStatus) -> bool {
        match self.status {
            RunbookBatchStatus::Idle => {
                return next == RunbookBatchStatus::Warming || next == RunbookBatchStatus::Draining;
            }
            RunbookBatchStatus::Warming => {
                return next == RunbookBatchStatus::Draining || next == RunbookBatchStatus::Stopped;
            }
            RunbookBatchStatus::Ready => {
                return next == RunbookBatchStatus::Draining || next == RunbookBatchStatus::Stopped;
            }
            RunbookBatchStatus::Draining => {
                return next == RunbookBatchStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "runbook batch", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.expires_at as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookBatch {
        RunbookBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            floor_value: self.floor_value,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a runbook batch in status idle.
pub fn new_runbook_batch(code: &str, title: &str) -> RunbookBatch {
    RunbookBatch {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        floor_value: 0.0,
        active: false,
        status: RunbookBatchStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_runbook_batch("code-112", "title-126");
        assert_eq!(item.status, RunbookBatchStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_runbook_batch("code-112", "title-126");
        if item.move_to(RunbookBatchStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookBatchStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_runbook_batch("code-112", "title-126");
        item.status = RunbookBatchStatus::Stopped;
        assert!(item.move_to(RunbookBatchStatus::Idle).is_err(), "expected move to fail");
    }
}
