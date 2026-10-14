use super::*;

use acme_kit::text;

/// Where a batched failover score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedFailoverScoreStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedFailoverScoreStatus {
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

impl std::fmt::Display for BatchedFailoverScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched failover score as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverScore {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub burst: i64,
    pub label: String,
    pub contact_email: String,
    pub priority: i64,
    pub cipher: String,
    pub status: BatchedFailoverScoreStatus,
}

impl BatchedFailoverScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedFailoverScoreStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedFailoverScoreStatus) -> bool {
        match self.status {
            BatchedFailoverScoreStatus::Planned => {
                match next {
                    BatchedFailoverScoreStatus::InProgress | BatchedFailoverScoreStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedFailoverScoreStatus::Scheduled => {
                return next == BatchedFailoverScoreStatus::InProgress;
            }
            BatchedFailoverScoreStatus::InProgress => {
                match next {
                    BatchedFailoverScoreStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedFailoverScoreStatus::Completed => {
                return next == BatchedFailoverScoreStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedFailoverScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched failover score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.priority as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedFailoverScore {
        BatchedFailoverScore {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            burst: self.burst,
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            priority: self.priority,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched failover score in status planned.
pub fn new_batched_failover_score(code: &str, title: &str) -> BatchedFailoverScore {
    BatchedFailoverScore {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        burst: 0,
        label: "".to_string(),
        contact_email: "".to_string(),
        priority: 0,
        cipher: "".to_string(),
        status: BatchedFailoverScoreStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_batched_failover_score("code-367", "title-416");
        assert_eq!(item.status, BatchedFailoverScoreStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_batched_failover_score("code-367", "title-416");
        if item.move_to(BatchedFailoverScoreStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedFailoverScoreStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_batched_failover_score("code-367", "title-416");
        item.status = BatchedFailoverScoreStatus::Cancelled;
        assert!(
            item.move_to(BatchedFailoverScoreStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
