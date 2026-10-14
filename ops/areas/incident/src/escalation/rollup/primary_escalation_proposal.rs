use super::*;

use acme_kit::text;

/// Where a primary escalation proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryEscalationProposalStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryEscalationProposalStatus {
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

impl std::fmt::Display for PrimaryEscalationProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary escalation proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationProposal {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub grace_days: i64,
    pub amount: f64,
    pub ceiling: f64,
    pub capacity: i64,
    pub fingerprint: String,
    pub audience: String,
    pub cost: f64,
    pub status: PrimaryEscalationProposalStatus,
}

impl PrimaryEscalationProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryEscalationProposalStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryEscalationProposalStatus) -> bool {
        match self.status {
            PrimaryEscalationProposalStatus::Open => {
                return next == PrimaryEscalationProposalStatus::Archived;
            }
            PrimaryEscalationProposalStatus::InReview => {
                return next == PrimaryEscalationProposalStatus::Reopened
                    || next == PrimaryEscalationProposalStatus::Archived;
            }
            PrimaryEscalationProposalStatus::Resolved => {
                match next {
                    PrimaryEscalationProposalStatus::Reopened | PrimaryEscalationProposalStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEscalationProposalStatus::Reopened => {
                return next == PrimaryEscalationProposalStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryEscalationProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary escalation proposal {} cannot move from {} to {}",
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
        format!("{} {}", "primary escalation proposal", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.expires_at as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryEscalationProposal {
        PrimaryEscalationProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            grace_days: self.grace_days,
            amount: self.amount,
            ceiling: self.ceiling,
            capacity: self.capacity,
            fingerprint: self.fingerprint.clone(),
            audience: self.audience.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a primary escalation proposal in status open.
pub fn new_primary_escalation_proposal(code: &str, name: &str) -> PrimaryEscalationProposal {
    PrimaryEscalationProposal {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        grace_days: 0,
        amount: 0.0,
        ceiling: 0.0,
        capacity: 0,
        fingerprint: "".to_string(),
        audience: "".to_string(),
        cost: 0.0,
        status: PrimaryEscalationProposalStatus::Open,
    }
}
