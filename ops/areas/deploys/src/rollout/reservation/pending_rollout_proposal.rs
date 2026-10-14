use super::*;

use acme_kit::text;

/// Where a pending rollout proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRolloutProposalStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PendingRolloutProposalStatus {
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

impl std::fmt::Display for PendingRolloutProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending rollout proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRolloutProposal {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub expires_at: i64,
    pub source: String,
    pub status: PendingRolloutProposalStatus,
}

impl PendingRolloutProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRolloutProposalStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRolloutProposalStatus) -> bool {
        match self.status {
            PendingRolloutProposalStatus::Planned => {
                match next {
                    PendingRolloutProposalStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRolloutProposalStatus::Scheduled => {
                return next == PendingRolloutProposalStatus::Completed;
            }
            PendingRolloutProposalStatus::InProgress => {
                match next {
                    PendingRolloutProposalStatus::Completed | PendingRolloutProposalStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRolloutProposalStatus::Completed => {
                return next == PendingRolloutProposalStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRolloutProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending rollout proposal {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.expires_at as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRolloutProposal {
        PendingRolloutProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            expires_at: self.expires_at,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending rollout proposal in status planned.
pub fn new_pending_rollout_proposal(code: &str, label: &str) -> PendingRolloutProposal {
    PendingRolloutProposal {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        expires_at: 0,
        source: "".to_string(),
        status: PendingRolloutProposalStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_pending_rollout_proposal("code-175", "label-562");
        assert_eq!(item.status, PendingRolloutProposalStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_pending_rollout_proposal("code-175", "label-562");
        if item.move_to(PendingRolloutProposalStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRolloutProposalStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_pending_rollout_proposal("code-175", "label-562");
        item.status = PendingRolloutProposalStatus::Cancelled;
        assert!(
            item.move_to(PendingRolloutProposalStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
