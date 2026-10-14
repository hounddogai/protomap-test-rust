use super::*;

use acme_kit::text;

/// Where a pending release proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingReleaseProposalStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingReleaseProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingReleaseProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending release proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseProposal {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub unit: String,
    pub phone: String,
    pub amount: f64,
    pub status: PendingReleaseProposalStatus,
}

impl PendingReleaseProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingReleaseProposalStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingReleaseProposalStatus) -> bool {
        match self.status {
            PendingReleaseProposalStatus::Proposed => {
                return next == PendingReleaseProposalStatus::Rejected;
            }
            PendingReleaseProposalStatus::Approved => {
                match next {
                    PendingReleaseProposalStatus::Rejected | PendingReleaseProposalStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingReleaseProposalStatus::Rejected => {
                return next == PendingReleaseProposalStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingReleaseProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending release proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> PendingReleaseProposal {
        PendingReleaseProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            unit: self.unit.clone(),
            phone: self.phone.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a pending release proposal in status proposed.
pub fn new_pending_release_proposal(code: &str, label: &str) -> PendingReleaseProposal {
    PendingReleaseProposal {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        unit: "".to_string(),
        phone: "".to_string(),
        amount: 0.0,
        status: PendingReleaseProposalStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_pending_release_proposal("code-394", "label-314");
        assert_eq!(item.status, PendingReleaseProposalStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_pending_release_proposal("code-394", "label-314");
        if item.move_to(PendingReleaseProposalStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingReleaseProposalStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_pending_release_proposal("code-394", "label-314");
        item.status = PendingReleaseProposalStatus::Withdrawn;
        assert!(
            item.move_to(PendingReleaseProposalStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
