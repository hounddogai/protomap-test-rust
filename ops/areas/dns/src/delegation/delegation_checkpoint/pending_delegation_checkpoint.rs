use super::*;

use acme_kit::text;

/// Where a pending delegation checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingDelegationCheckpointStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingDelegationCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingDelegationCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending delegation checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingDelegationCheckpoint {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub maximum: f64,
    pub budget: f64,
    pub summary: String,
    pub currency: String,
    pub archived: bool,
    pub external_ref: String,
    pub sequence: i64,
    pub version: i64,
    pub status: PendingDelegationCheckpointStatus,
}

impl PendingDelegationCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingDelegationCheckpointStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingDelegationCheckpointStatus) -> bool {
        match self.status {
            PendingDelegationCheckpointStatus::Proposed => {
                return next == PendingDelegationCheckpointStatus::Rejected;
            }
            PendingDelegationCheckpointStatus::Approved => {
                match next {
                    PendingDelegationCheckpointStatus::Rejected | PendingDelegationCheckpointStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingDelegationCheckpointStatus::Rejected => {
                return next == PendingDelegationCheckpointStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingDelegationCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending delegation checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending delegation checkpoint", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.budget * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingDelegationCheckpoint {
        PendingDelegationCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            maximum: self.maximum,
            budget: self.budget,
            summary: self.summary.clone(),
            currency: self.currency.clone(),
            archived: self.archived,
            external_ref: self.external_ref.clone(),
            sequence: self.sequence,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending delegation checkpoint in status proposed.
pub fn new_pending_delegation_checkpoint(code: &str, title: &str) -> PendingDelegationCheckpoint {
    PendingDelegationCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        maximum: 0.0,
        budget: 0.0,
        summary: "".to_string(),
        currency: "".to_string(),
        archived: false,
        external_ref: "".to_string(),
        sequence: 0,
        version: 0,
        status: PendingDelegationCheckpointStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_pending_delegation_checkpoint("code-667", "title-856");
        assert_eq!(item.status, PendingDelegationCheckpointStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_pending_delegation_checkpoint("code-667", "title-856");
        if item.move_to(PendingDelegationCheckpointStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingDelegationCheckpointStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_pending_delegation_checkpoint("code-667", "title-856");
        item.status = PendingDelegationCheckpointStatus::Withdrawn;
        assert!(
            item.move_to(PendingDelegationCheckpointStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
