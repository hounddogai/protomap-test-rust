use super::*;

use acme_kit::text;

/// Where a usage review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageReviewStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl UsageReviewStatus {
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

impl std::fmt::Display for UsageReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One usage review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageReview {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub summary: String,
    pub ends_at: i64,
    pub status: UsageReviewStatus,
}

impl UsageReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageReviewStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageReviewStatus) -> bool {
        match self.status {
            UsageReviewStatus::Open => {
                match next {
                    UsageReviewStatus::Resolved | UsageReviewStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageReviewStatus::InReview => {
                return next == UsageReviewStatus::Reopened || next == UsageReviewStatus::Archived;
            }
            UsageReviewStatus::Resolved => {
                return next == UsageReviewStatus::Reopened || next == UsageReviewStatus::Archived;
            }
            UsageReviewStatus::Reopened => {
                return next == UsageReviewStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage review {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.expires_at as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageReview {
        UsageReview {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            summary: self.summary.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a usage review in status open.
pub fn new_usage_review(code: &str, label: &str) -> UsageReview {
    UsageReview {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        summary: "".to_string(),
        ends_at: 0,
        status: UsageReviewStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_usage_review("code-715", "label-536");
        assert_eq!(item.status, UsageReviewStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_usage_review("code-715", "label-536");
        if item.move_to(UsageReviewStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageReviewStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_usage_review("code-715", "label-536");
        item.status = UsageReviewStatus::Archived;
        assert!(item.move_to(UsageReviewStatus::Open).is_err(), "expected move to fail");
    }
}
