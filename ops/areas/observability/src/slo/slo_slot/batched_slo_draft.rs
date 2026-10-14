use super::*;

use acme_kit::text;

/// Where a batched slo draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedSloDraftStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedSloDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for BatchedSloDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched slo draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSloDraft {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub description: String,
    pub score: f64,
    pub attempts: i64,
    pub currency: String,
    pub cluster: String,
    pub category: String,
    pub status: BatchedSloDraftStatus,
}

impl BatchedSloDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedSloDraftStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedSloDraftStatus) -> bool {
        match self.status {
            BatchedSloDraftStatus::Planned => {
                match next {
                    BatchedSloDraftStatus::Completed | BatchedSloDraftStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSloDraftStatus::Scheduled => {
                match next {
                    BatchedSloDraftStatus::InProgress | BatchedSloDraftStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSloDraftStatus::InProgress => {
                match next {
                    BatchedSloDraftStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSloDraftStatus::Completed => {
                match next {
                    BatchedSloDraftStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: BatchedSloDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched slo draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.attempts as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedSloDraft {
        BatchedSloDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            description: self.description.clone(),
            score: self.score,
            attempts: self.attempts,
            currency: self.currency.clone(),
            cluster: self.cluster.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a batched slo draft in status planned.
pub fn new_batched_slo_draft(code: &str, name: &str) -> BatchedSloDraft {
    BatchedSloDraft {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        description: "".to_string(),
        score: 0.0,
        attempts: 0,
        currency: "".to_string(),
        cluster: "".to_string(),
        category: "".to_string(),
        status: BatchedSloDraftStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_batched_slo_draft("code-269", "name-160");
        assert_eq!(item.status, BatchedSloDraftStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_batched_slo_draft("code-269", "name-160");
        if item.move_to(BatchedSloDraftStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedSloDraftStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_batched_slo_draft("code-269", "name-160");
        item.status = BatchedSloDraftStatus::Cancelled;
        assert!(
            item.move_to(BatchedSloDraftStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
