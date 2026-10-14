use super::*;

use acme_kit::text;

/// Where a retention exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionExceptionStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RetentionExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RetentionExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One retention exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionException {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub attempts: i64,
    pub version: i64,
    pub archived: bool,
    pub ceiling: f64,
    pub status: RetentionExceptionStatus,
}

impl RetentionException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionExceptionStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionExceptionStatus) -> bool {
        match self.status {
            RetentionExceptionStatus::Requested => {
                match next {
                    RetentionExceptionStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionExceptionStatus::Granted => {
                match next {
                    RetentionExceptionStatus::Revoked | RetentionExceptionStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionExceptionStatus::Revoked => {
                return next == RetentionExceptionStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.cost + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionException {
        RetentionException {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            attempts: self.attempts,
            version: self.version,
            archived: self.archived,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a retention exception in status requested.
pub fn new_retention_exception(code: &str, title: &str) -> RetentionException {
    RetentionException {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        attempts: 0,
        version: 0,
        archived: false,
        ceiling: 0.0,
        status: RetentionExceptionStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_retention_exception("code-254", "title-506");
        assert_eq!(item.status, RetentionExceptionStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_retention_exception("code-254", "title-506");
        if item.move_to(RetentionExceptionStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionExceptionStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_retention_exception("code-254", "title-506");
        item.status = RetentionExceptionStatus::Expired;
        assert!(
            item.move_to(RetentionExceptionStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
