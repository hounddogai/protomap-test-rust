use super::*;

use acme_kit::text;

/// Where a lenient freeze checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientFreezeCheckpointStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientFreezeCheckpointStatus {
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

impl std::fmt::Display for LenientFreezeCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient freeze checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeCheckpoint {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub port: i64,
    pub namespace: String,
    pub minimum: f64,
    pub amount: f64,
    pub owner_id: String,
    pub summary: String,
    pub category: String,
    pub attempts: i64,
    pub status: LenientFreezeCheckpointStatus,
}

impl LenientFreezeCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientFreezeCheckpointStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientFreezeCheckpointStatus) -> bool {
        match self.status {
            LenientFreezeCheckpointStatus::Planned => {
                return next == LenientFreezeCheckpointStatus::Cancelled;
            }
            LenientFreezeCheckpointStatus::Scheduled => {
                match next {
                    LenientFreezeCheckpointStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientFreezeCheckpointStatus::InProgress => {
                match next {
                    LenientFreezeCheckpointStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientFreezeCheckpointStatus::Completed => {
                return next == LenientFreezeCheckpointStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientFreezeCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient freeze checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.minimum + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientFreezeCheckpoint {
        LenientFreezeCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            port: self.port,
            namespace: self.namespace.clone(),
            minimum: self.minimum,
            amount: self.amount,
            owner_id: self.owner_id.clone(),
            summary: self.summary.clone(),
            category: self.category.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a lenient freeze checkpoint in status planned.
pub fn new_lenient_freeze_checkpoint(code: &str, name: &str) -> LenientFreezeCheckpoint {
    LenientFreezeCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        port: 0,
        namespace: "".to_string(),
        minimum: 0.0,
        amount: 0.0,
        owner_id: "".to_string(),
        summary: "".to_string(),
        category: "".to_string(),
        attempts: 0,
        status: LenientFreezeCheckpointStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_freeze_checkpoint("code-914", "name-783");
        assert_eq!(item.status, LenientFreezeCheckpointStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_lenient_freeze_checkpoint("code-914", "name-783");
        if item.move_to(LenientFreezeCheckpointStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientFreezeCheckpointStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_freeze_checkpoint("code-914", "name-783");
        item.status = LenientFreezeCheckpointStatus::Cancelled;
        assert!(
            item.move_to(LenientFreezeCheckpointStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
