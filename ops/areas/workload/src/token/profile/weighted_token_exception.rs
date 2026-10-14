use super::*;

use acme_kit::text;

/// Where a weighted token exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedTokenExceptionStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl WeightedTokenExceptionStatus {
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

impl std::fmt::Display for WeightedTokenExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted token exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenException {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub minimum: f64,
    pub priority: i64,
    pub revision: i64,
    pub scope: String,
    pub description: String,
    pub ends_at: i64,
    pub status: WeightedTokenExceptionStatus,
}

impl WeightedTokenException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedTokenExceptionStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedTokenExceptionStatus) -> bool {
        match self.status {
            WeightedTokenExceptionStatus::Open => {
                match next {
                    WeightedTokenExceptionStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTokenExceptionStatus::InReview => {
                match next {
                    WeightedTokenExceptionStatus::Reopened | WeightedTokenExceptionStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTokenExceptionStatus::Resolved => {
                match next {
                    WeightedTokenExceptionStatus::Reopened | WeightedTokenExceptionStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTokenExceptionStatus::Reopened => {
                return next == WeightedTokenExceptionStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedTokenExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted token exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.batch_size as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedTokenException {
        WeightedTokenException {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            minimum: self.minimum,
            priority: self.priority,
            revision: self.revision,
            scope: self.scope.clone(),
            description: self.description.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a weighted token exception in status open.
pub fn new_weighted_token_exception(code: &str, label: &str) -> WeightedTokenException {
    WeightedTokenException {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        minimum: 0.0,
        priority: 0,
        revision: 0,
        scope: "".to_string(),
        description: "".to_string(),
        ends_at: 0,
        status: WeightedTokenExceptionStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_weighted_token_exception("code-280", "label-542");
        assert_eq!(item.status, WeightedTokenExceptionStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_weighted_token_exception("code-280", "label-542");
        if item.move_to(WeightedTokenExceptionStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedTokenExceptionStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_weighted_token_exception("code-280", "label-542");
        item.status = WeightedTokenExceptionStatus::Archived;
        assert!(
            item.move_to(WeightedTokenExceptionStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
