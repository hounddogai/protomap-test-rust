use super::*;

use acme_kit::text;

/// Where a runbook group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookGroupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RunbookGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RunbookGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook group as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookGroup {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub floor_value: f64,
    pub cost: f64,
    pub status: RunbookGroupStatus,
}

impl RunbookGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookGroupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookGroupStatus) -> bool {
        match self.status {
            RunbookGroupStatus::Open => {
                match next {
                    RunbookGroupStatus::InReview | RunbookGroupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookGroupStatus::InReview => {
                return next == RunbookGroupStatus::Reopened || next == RunbookGroupStatus::Archived;
            }
            RunbookGroupStatus::Resolved => {
                match next {
                    RunbookGroupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookGroupStatus::Reopened => {
                match next {
                    RunbookGroupStatus::Archived => {
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

    pub fn move_to(&mut self, next: RunbookGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook group {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.floor_value - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookGroup {
        RunbookGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            created_at: self.created_at,
            expires_at: self.expires_at,
            floor_value: self.floor_value,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a runbook group in status open.
pub fn new_runbook_group(code: &str, title: &str) -> RunbookGroup {
    RunbookGroup {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        created_at: 0,
        expires_at: 0,
        floor_value: 0.0,
        cost: 0.0,
        status: RunbookGroupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_runbook_group("code-832", "title-134");
        assert_eq!(item.status, RunbookGroupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_runbook_group("code-832", "title-134");
        if item.move_to(RunbookGroupStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookGroupStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_runbook_group("code-832", "title-134");
        item.status = RunbookGroupStatus::Archived;
        assert!(item.move_to(RunbookGroupStatus::Open).is_err(), "expected move to fail");
    }
}
