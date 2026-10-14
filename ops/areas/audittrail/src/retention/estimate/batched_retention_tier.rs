use super::*;

use acme_kit::text;

/// Where a batched retention tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRetentionTierStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedRetentionTierStatus {
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

impl std::fmt::Display for BatchedRetentionTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched retention tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionTier {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub threshold: f64,
    pub summary: String,
    pub cluster: String,
    pub status: BatchedRetentionTierStatus,
}

impl BatchedRetentionTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRetentionTierStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRetentionTierStatus) -> bool {
        match self.status {
            BatchedRetentionTierStatus::Open => {
                match next {
                    BatchedRetentionTierStatus::InReview | BatchedRetentionTierStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRetentionTierStatus::InReview => {
                match next {
                    BatchedRetentionTierStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRetentionTierStatus::Resolved => {
                match next {
                    BatchedRetentionTierStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRetentionTierStatus::Reopened => {
                match next {
                    BatchedRetentionTierStatus::Archived => {
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

    pub fn move_to(&mut self, next: BatchedRetentionTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched retention tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched retention tier", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        (self.threshold * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRetentionTier {
        BatchedRetentionTier {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            threshold: self.threshold,
            summary: self.summary.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a batched retention tier in status open.
pub fn new_batched_retention_tier(code: &str, name: &str) -> BatchedRetentionTier {
    BatchedRetentionTier {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        threshold: 0.0,
        summary: "".to_string(),
        cluster: "".to_string(),
        status: BatchedRetentionTierStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_batched_retention_tier("code-854", "name-142");
        assert_eq!(item.status, BatchedRetentionTierStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_batched_retention_tier("code-854", "name-142");
        if item.move_to(BatchedRetentionTierStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRetentionTierStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_batched_retention_tier("code-854", "name-142");
        item.status = BatchedRetentionTierStatus::Archived;
        assert!(
            item.move_to(BatchedRetentionTierStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
