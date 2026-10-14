use super::*;

use acme_kit::text;

/// Where a runbook adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RunbookAdjustmentStatus {
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

impl std::fmt::Display for RunbookAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One runbook adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAdjustment {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub grace_days: i64,
    pub issuer: String,
    pub status: RunbookAdjustmentStatus,
}

impl RunbookAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookAdjustmentStatus) -> bool {
        match self.status {
            RunbookAdjustmentStatus::New => {
                return next == RunbookAdjustmentStatus::Failed;
            }
            RunbookAdjustmentStatus::Queued => {
                return next == RunbookAdjustmentStatus::Failed;
            }
            RunbookAdjustmentStatus::Running => {
                match next {
                    RunbookAdjustmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookAdjustmentStatus::Done => {
                return next == RunbookAdjustmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.grace_days as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookAdjustment {
        RunbookAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            grace_days: self.grace_days,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a runbook adjustment in status new.
pub fn new_runbook_adjustment(code: &str, title: &str) -> RunbookAdjustment {
    RunbookAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        grace_days: 0,
        issuer: "".to_string(),
        status: RunbookAdjustmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_runbook_adjustment("code-166", "title-571");
        assert_eq!(item.status, RunbookAdjustmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_runbook_adjustment("code-166", "title-571");
        if item.move_to(RunbookAdjustmentStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookAdjustmentStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_runbook_adjustment("code-166", "title-571");
        item.status = RunbookAdjustmentStatus::Failed;
        assert!(
            item.move_to(RunbookAdjustmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
