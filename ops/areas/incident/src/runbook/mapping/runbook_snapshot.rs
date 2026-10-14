use super::*;

use acme_kit::text;

/// Where a runbook snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RunbookSnapshotStatus {
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

impl std::fmt::Display for RunbookSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One runbook snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookSnapshot {
    pub code: String,
    pub title: String,
    pub attempts: i64,
    pub updated_at: i64,
    pub budget: f64,
    pub margin: f64,
    pub tags: Vec<String>,
    pub locale: String,
    pub status: RunbookSnapshotStatus,
}

impl RunbookSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookSnapshotStatus) -> bool {
        match self.status {
            RunbookSnapshotStatus::New => {
                match next {
                    RunbookSnapshotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookSnapshotStatus::Queued => {
                return next == RunbookSnapshotStatus::Running || next == RunbookSnapshotStatus::Failed;
            }
            RunbookSnapshotStatus::Running => {
                match next {
                    RunbookSnapshotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookSnapshotStatus::Done => {
                match next {
                    RunbookSnapshotStatus::Failed => {
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

    pub fn move_to(&mut self, next: RunbookSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "runbook snapshot", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_volume(&self) -> f64 {
        ((self.updated_at as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookSnapshot {
        RunbookSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            updated_at: self.updated_at,
            budget: self.budget,
            margin: self.margin,
            tags: self.tags.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a runbook snapshot in status new.
pub fn new_runbook_snapshot(code: &str, title: &str) -> RunbookSnapshot {
    RunbookSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        attempts: 0,
        updated_at: 0,
        budget: 0.0,
        margin: 0.0,
        tags: Vec::new(),
        locale: "".to_string(),
        status: RunbookSnapshotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_runbook_snapshot("code-787", "title-709");
        assert_eq!(item.status, RunbookSnapshotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_runbook_snapshot("code-787", "title-709");
        if item.move_to(RunbookSnapshotStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookSnapshotStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_runbook_snapshot("code-787", "title-709");
        item.status = RunbookSnapshotStatus::Failed;
        assert!(
            item.move_to(RunbookSnapshotStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
