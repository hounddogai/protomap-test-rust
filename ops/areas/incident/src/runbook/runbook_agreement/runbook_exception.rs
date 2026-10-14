use super::*;

use acme_kit::text;

/// Where a runbook exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookExceptionStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RunbookExceptionStatus {
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

impl std::fmt::Display for RunbookExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a runbook exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookException {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub starts_at: i64,
    pub attempts: i64,
    pub status: RunbookExceptionStatus,
}

impl RunbookException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookExceptionStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookExceptionStatus) -> bool {
        match self.status {
            RunbookExceptionStatus::Open => {
                match next {
                    RunbookExceptionStatus::InReview | RunbookExceptionStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookExceptionStatus::InReview => {
                return next == RunbookExceptionStatus::Resolved || next == RunbookExceptionStatus::Archived;
            }
            RunbookExceptionStatus::Resolved => {
                return next == RunbookExceptionStatus::Reopened || next == RunbookExceptionStatus::Archived;
            }
            RunbookExceptionStatus::Reopened => {
                match next {
                    RunbookExceptionStatus::Archived => {
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

    pub fn move_to(&mut self, next: RunbookExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.starts_at as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookException {
        RunbookException {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            starts_at: self.starts_at,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a runbook exception in status open.
pub fn new_runbook_exception(code: &str, title: &str) -> RunbookException {
    RunbookException {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        starts_at: 0,
        attempts: 0,
        status: RunbookExceptionStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_runbook_exception("code-871", "title-542");
        assert_eq!(item.status, RunbookExceptionStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_runbook_exception("code-871", "title-542");
        if item.move_to(RunbookExceptionStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookExceptionStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_runbook_exception("code-871", "title-542");
        item.status = RunbookExceptionStatus::Archived;
        assert!(
            item.move_to(RunbookExceptionStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
