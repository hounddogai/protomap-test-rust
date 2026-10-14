use super::*;

use acme_kit::text;

/// Where a batched failover review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedFailoverReviewStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedFailoverReviewStatus {
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

impl std::fmt::Display for BatchedFailoverReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched failover review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverReview {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub label: String,
    pub ratio: f64,
    pub host: String,
    pub cost: f64,
    pub approved: bool,
    pub scope: String,
    pub port: i64,
    pub status: BatchedFailoverReviewStatus,
}

impl BatchedFailoverReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedFailoverReviewStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedFailoverReviewStatus) -> bool {
        match self.status {
            BatchedFailoverReviewStatus::Draft => {
                return next == BatchedFailoverReviewStatus::Active;
            }
            BatchedFailoverReviewStatus::Pending => {
                match next {
                    BatchedFailoverReviewStatus::Active | BatchedFailoverReviewStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedFailoverReviewStatus::Active => {
                match next {
                    BatchedFailoverReviewStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedFailoverReviewStatus::Suspended => {
                match next {
                    BatchedFailoverReviewStatus::Closed => {
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

    pub fn move_to(&mut self, next: BatchedFailoverReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched failover review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.rate + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedFailoverReview {
        BatchedFailoverReview {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            label: self.label.clone(),
            ratio: self.ratio,
            host: self.host.clone(),
            cost: self.cost,
            approved: self.approved,
            scope: self.scope.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a batched failover review in status draft.
pub fn new_batched_failover_review(code: &str, name: &str) -> BatchedFailoverReview {
    BatchedFailoverReview {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        label: "".to_string(),
        ratio: 0.0,
        host: "".to_string(),
        cost: 0.0,
        approved: false,
        scope: "".to_string(),
        port: 0,
        status: BatchedFailoverReviewStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_failover_review("code-936", "name-571");
        assert_eq!(item.status, BatchedFailoverReviewStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_batched_failover_review("code-936", "name-571");
        if item.move_to(BatchedFailoverReviewStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedFailoverReviewStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_failover_review("code-936", "name-571");
        item.status = BatchedFailoverReviewStatus::Closed;
        assert!(
            item.move_to(BatchedFailoverReviewStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
