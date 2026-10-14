use super::*;

use acme_kit::text;

/// Where a batched nonce review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedNonceReviewStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedNonceReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for BatchedNonceReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched nonce review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedNonceReview {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub archived: bool,
    pub weight: f64,
    pub status: BatchedNonceReviewStatus,
}

impl BatchedNonceReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedNonceReviewStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedNonceReviewStatus) -> bool {
        match self.status {
            BatchedNonceReviewStatus::Draft => {
                match next {
                    BatchedNonceReviewStatus::Pending | BatchedNonceReviewStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedNonceReviewStatus::Pending => {
                return next == BatchedNonceReviewStatus::Active || next == BatchedNonceReviewStatus::Suspended;
            }
            BatchedNonceReviewStatus::Active => {
                return next == BatchedNonceReviewStatus::Suspended;
            }
            BatchedNonceReviewStatus::Suspended => {
                return next == BatchedNonceReviewStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedNonceReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched nonce review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        (self.margin * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedNonceReview {
        BatchedNonceReview {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            archived: self.archived,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched nonce review in status draft.
pub fn new_batched_nonce_review(code: &str, name: &str) -> BatchedNonceReview {
    BatchedNonceReview {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        archived: false,
        weight: 0.0,
        status: BatchedNonceReviewStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_nonce_review("code-337", "name-847");
        assert_eq!(item.status, BatchedNonceReviewStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_batched_nonce_review("code-337", "name-847");
        if item.move_to(BatchedNonceReviewStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedNonceReviewStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_nonce_review("code-337", "name-847");
        item.status = BatchedNonceReviewStatus::Closed;
        assert!(
            item.move_to(BatchedNonceReviewStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
