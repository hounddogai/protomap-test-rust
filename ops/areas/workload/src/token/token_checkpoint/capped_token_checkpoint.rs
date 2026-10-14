use super::*;

use acme_kit::text;

/// Where a capped token checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedTokenCheckpointStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CappedTokenCheckpointStatus {
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

impl std::fmt::Display for CappedTokenCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped token checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenCheckpoint {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub expires_at: i64,
    pub weight: f64,
    pub contact_email: String,
    pub threshold: f64,
    pub rate: f64,
    pub capacity: i64,
    pub limit: i64,
    pub status: CappedTokenCheckpointStatus,
}

impl CappedTokenCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedTokenCheckpointStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedTokenCheckpointStatus) -> bool {
        match self.status {
            CappedTokenCheckpointStatus::Planned => {
                return next == CappedTokenCheckpointStatus::InProgress;
            }
            CappedTokenCheckpointStatus::Scheduled => {
                return next == CappedTokenCheckpointStatus::Completed
                    || next == CappedTokenCheckpointStatus::Cancelled;
            }
            CappedTokenCheckpointStatus::InProgress => {
                match next {
                    CappedTokenCheckpointStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedTokenCheckpointStatus::Completed => {
                return next == CappedTokenCheckpointStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedTokenCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped token checkpoint {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.limit as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedTokenCheckpoint {
        CappedTokenCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            expires_at: self.expires_at,
            weight: self.weight,
            contact_email: self.contact_email.clone(),
            threshold: self.threshold,
            rate: self.rate,
            capacity: self.capacity,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped token checkpoint in status planned.
pub fn new_capped_token_checkpoint(code: &str, title: &str) -> CappedTokenCheckpoint {
    CappedTokenCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        expires_at: 0,
        weight: 0.0,
        contact_email: "".to_string(),
        threshold: 0.0,
        rate: 0.0,
        capacity: 0,
        limit: 0,
        status: CappedTokenCheckpointStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_capped_token_checkpoint("code-132", "title-77");
        assert_eq!(item.status, CappedTokenCheckpointStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_capped_token_checkpoint("code-132", "title-77");
        if item.move_to(CappedTokenCheckpointStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedTokenCheckpointStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_capped_token_checkpoint("code-132", "title-77");
        item.status = CappedTokenCheckpointStatus::Cancelled;
        assert!(
            item.move_to(CappedTokenCheckpointStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
