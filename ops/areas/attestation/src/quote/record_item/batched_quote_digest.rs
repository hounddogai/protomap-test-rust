use super::*;

use acme_kit::text;

/// Where a batched quote digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedQuoteDigestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedQuoteDigestStatus {
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

impl std::fmt::Display for BatchedQuoteDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched quote digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteDigest {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub category: String,
    pub ttl_seconds: i64,
    pub owner_id: String,
    pub pinned: bool,
    pub window_days: i64,
    pub ceiling: f64,
    pub status: BatchedQuoteDigestStatus,
}

impl BatchedQuoteDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedQuoteDigestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedQuoteDigestStatus) -> bool {
        match self.status {
            BatchedQuoteDigestStatus::Open => {
                match next {
                    BatchedQuoteDigestStatus::InReview | BatchedQuoteDigestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedQuoteDigestStatus::InReview => {
                return next == BatchedQuoteDigestStatus::Reopened;
            }
            BatchedQuoteDigestStatus::Resolved => {
                match next {
                    BatchedQuoteDigestStatus::Reopened | BatchedQuoteDigestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedQuoteDigestStatus::Reopened => {
                return next == BatchedQuoteDigestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedQuoteDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched quote digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        (self.ceiling * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedQuoteDigest {
        BatchedQuoteDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            category: self.category.clone(),
            ttl_seconds: self.ttl_seconds,
            owner_id: self.owner_id.clone(),
            pinned: self.pinned,
            window_days: self.window_days,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a batched quote digest in status open.
pub fn new_batched_quote_digest(code: &str, name: &str) -> BatchedQuoteDigest {
    BatchedQuoteDigest {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        category: "".to_string(),
        ttl_seconds: 0,
        owner_id: "".to_string(),
        pinned: false,
        window_days: 0,
        ceiling: 0.0,
        status: BatchedQuoteDigestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_batched_quote_digest("code-557", "name-992");
        assert_eq!(item.status, BatchedQuoteDigestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_batched_quote_digest("code-557", "name-992");
        if item.move_to(BatchedQuoteDigestStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedQuoteDigestStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_batched_quote_digest("code-557", "name-992");
        item.status = BatchedQuoteDigestStatus::Archived;
        assert!(
            item.move_to(BatchedQuoteDigestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
