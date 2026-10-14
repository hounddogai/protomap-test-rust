use super::*;

use acme_kit::text;

/// Where a batched retention override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRetentionOverrideStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedRetentionOverrideStatus {
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

impl std::fmt::Display for BatchedRetentionOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched retention override as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionOverride {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub starts_at: i64,
    pub ttl_seconds: i64,
    pub key_id: String,
    pub status: BatchedRetentionOverrideStatus,
}

impl BatchedRetentionOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRetentionOverrideStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRetentionOverrideStatus) -> bool {
        match self.status {
            BatchedRetentionOverrideStatus::Open => {
                return next == BatchedRetentionOverrideStatus::Resolved;
            }
            BatchedRetentionOverrideStatus::InReview => {
                return next == BatchedRetentionOverrideStatus::Resolved;
            }
            BatchedRetentionOverrideStatus::Resolved => {
                return next == BatchedRetentionOverrideStatus::Reopened
                    || next == BatchedRetentionOverrideStatus::Archived;
            }
            BatchedRetentionOverrideStatus::Reopened => {
                return next == BatchedRetentionOverrideStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedRetentionOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched retention override {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRetentionOverride {
        BatchedRetentionOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            starts_at: self.starts_at,
            ttl_seconds: self.ttl_seconds,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a batched retention override in status open.
pub fn new_batched_retention_override(code: &str, title: &str) -> BatchedRetentionOverride {
    BatchedRetentionOverride {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        starts_at: 0,
        ttl_seconds: 0,
        key_id: "".to_string(),
        status: BatchedRetentionOverrideStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_batched_retention_override("code-193", "title-178");
        assert_eq!(item.status, BatchedRetentionOverrideStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_batched_retention_override("code-193", "title-178");
        if item.move_to(BatchedRetentionOverrideStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRetentionOverrideStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_batched_retention_override("code-193", "title-178");
        item.status = BatchedRetentionOverrideStatus::Archived;
        assert!(
            item.move_to(BatchedRetentionOverrideStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
