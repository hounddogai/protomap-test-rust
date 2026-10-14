use super::*;

use acme_kit::text;

/// Where a batched runbook variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRunbookVarianceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedRunbookVarianceStatus {
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

impl std::fmt::Display for BatchedRunbookVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched runbook variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookVariance {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub category: String,
    pub sequence: i64,
    pub score: f64,
    pub revision: i64,
    pub status: BatchedRunbookVarianceStatus,
}

impl BatchedRunbookVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRunbookVarianceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRunbookVarianceStatus) -> bool {
        match self.status {
            BatchedRunbookVarianceStatus::Open => {
                return next == BatchedRunbookVarianceStatus::Resolved;
            }
            BatchedRunbookVarianceStatus::InReview => {
                return next == BatchedRunbookVarianceStatus::Archived;
            }
            BatchedRunbookVarianceStatus::Resolved => {
                match next {
                    BatchedRunbookVarianceStatus::Reopened | BatchedRunbookVarianceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRunbookVarianceStatus::Reopened => {
                match next {
                    BatchedRunbookVarianceStatus::Archived => {
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

    pub fn move_to(&mut self, next: BatchedRunbookVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched runbook variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.sequence as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRunbookVariance {
        BatchedRunbookVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            category: self.category.clone(),
            sequence: self.sequence,
            score: self.score,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a batched runbook variance in status open.
pub fn new_batched_runbook_variance(code: &str, label: &str) -> BatchedRunbookVariance {
    BatchedRunbookVariance {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        category: "".to_string(),
        sequence: 0,
        score: 0.0,
        revision: 0,
        status: BatchedRunbookVarianceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_batched_runbook_variance("code-295", "label-627");
        assert_eq!(item.status, BatchedRunbookVarianceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_batched_runbook_variance("code-295", "label-627");
        if item.move_to(BatchedRunbookVarianceStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRunbookVarianceStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_batched_runbook_variance("code-295", "label-627");
        item.status = BatchedRunbookVarianceStatus::Archived;
        assert!(
            item.move_to(BatchedRunbookVarianceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
