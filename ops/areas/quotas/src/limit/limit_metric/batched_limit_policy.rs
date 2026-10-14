use super::*;

use acme_kit::text;

/// Where a batched limit policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedLimitPolicyStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedLimitPolicyStatus {
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

impl std::fmt::Display for BatchedLimitPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched limit policy as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitPolicy {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub minimum: f64,
    pub discount: f64,
    pub currency: String,
    pub score: f64,
    pub status: BatchedLimitPolicyStatus,
}

impl BatchedLimitPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedLimitPolicyStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedLimitPolicyStatus) -> bool {
        match self.status {
            BatchedLimitPolicyStatus::Open => {
                return next == BatchedLimitPolicyStatus::Reopened;
            }
            BatchedLimitPolicyStatus::InReview => {
                match next {
                    BatchedLimitPolicyStatus::Reopened | BatchedLimitPolicyStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedLimitPolicyStatus::Resolved => {
                match next {
                    BatchedLimitPolicyStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedLimitPolicyStatus::Reopened => {
                return next == BatchedLimitPolicyStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedLimitPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched limit policy {} cannot move from {} to {}",
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
        format!("{} {}", "batched limit policy", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        (self.discount * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedLimitPolicy {
        BatchedLimitPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            minimum: self.minimum,
            discount: self.discount,
            currency: self.currency.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched limit policy in status open.
pub fn new_batched_limit_policy(code: &str, name: &str) -> BatchedLimitPolicy {
    BatchedLimitPolicy {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        minimum: 0.0,
        discount: 0.0,
        currency: "".to_string(),
        score: 0.0,
        status: BatchedLimitPolicyStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_batched_limit_policy("code-740", "name-368");
        assert_eq!(item.status, BatchedLimitPolicyStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_batched_limit_policy("code-740", "name-368");
        if item.move_to(BatchedLimitPolicyStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedLimitPolicyStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_batched_limit_policy("code-740", "name-368");
        item.status = BatchedLimitPolicyStatus::Archived;
        assert!(
            item.move_to(BatchedLimitPolicyStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
