use super::*;

use acme_kit::text;

/// Where a rolling rollback exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingRollbackExceptionStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingRollbackExceptionStatus {
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

impl std::fmt::Display for RollingRollbackExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling rollback exception as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRollbackException {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub label: String,
    pub burst: i64,
    pub status: RollingRollbackExceptionStatus,
}

impl RollingRollbackException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingRollbackExceptionStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingRollbackExceptionStatus) -> bool {
        match self.status {
            RollingRollbackExceptionStatus::Open => {
                match next {
                    RollingRollbackExceptionStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRollbackExceptionStatus::InReview => {
                match next {
                    RollingRollbackExceptionStatus::Resolved | RollingRollbackExceptionStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRollbackExceptionStatus::Resolved => {
                match next {
                    RollingRollbackExceptionStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRollbackExceptionStatus::Reopened => {
                return next == RollingRollbackExceptionStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingRollbackExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling rollback exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.sequence as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingRollbackException {
        RollingRollbackException {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            label: self.label.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling rollback exception in status open.
pub fn new_rolling_rollback_exception(code: &str, title: &str) -> RollingRollbackException {
    RollingRollbackException {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        label: "".to_string(),
        burst: 0,
        status: RollingRollbackExceptionStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rolling_rollback_exception("code-354", "title-390");
        assert_eq!(item.status, RollingRollbackExceptionStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_rolling_rollback_exception("code-354", "title-390");
        if item.move_to(RollingRollbackExceptionStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingRollbackExceptionStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rolling_rollback_exception("code-354", "title-390");
        item.status = RollingRollbackExceptionStatus::Archived;
        assert!(
            item.move_to(RollingRollbackExceptionStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
