use super::*;

use acme_kit::text;

/// Where a rollout batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutBatchStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RolloutBatchStatus {
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

impl std::fmt::Display for RolloutBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rollout batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutBatch {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub expires_at: i64,
    pub label: String,
    pub cipher: String,
    pub capacity: i64,
    pub budget: f64,
    pub discount: f64,
    pub status: RolloutBatchStatus,
}

impl RolloutBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutBatchStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutBatchStatus) -> bool {
        match self.status {
            RolloutBatchStatus::Planned => {
                return next == RolloutBatchStatus::InProgress;
            }
            RolloutBatchStatus::Scheduled => {
                return next == RolloutBatchStatus::InProgress || next == RolloutBatchStatus::Cancelled;
            }
            RolloutBatchStatus::InProgress => {
                match next {
                    RolloutBatchStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutBatchStatus::Completed => {
                return next == RolloutBatchStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout batch {} cannot move from {} to {}",
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
        format!("{} {}", "rollout batch", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.budget + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutBatch {
        RolloutBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            expires_at: self.expires_at,
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            capacity: self.capacity,
            budget: self.budget,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rollout batch in status planned.
pub fn new_rollout_batch(code: &str, name: &str) -> RolloutBatch {
    RolloutBatch {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        expires_at: 0,
        label: "".to_string(),
        cipher: "".to_string(),
        capacity: 0,
        budget: 0.0,
        discount: 0.0,
        status: RolloutBatchStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rollout_batch("code-726", "name-746");
        assert_eq!(item.status, RolloutBatchStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_rollout_batch("code-726", "name-746");
        if item.move_to(RolloutBatchStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutBatchStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rollout_batch("code-726", "name-746");
        item.status = RolloutBatchStatus::Cancelled;
        assert!(
            item.move_to(RolloutBatchStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
