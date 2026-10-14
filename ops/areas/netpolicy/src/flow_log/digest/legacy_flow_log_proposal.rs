use super::*;

use acme_kit::text;

/// Where a legacy flow log proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyFlowLogProposalStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyFlowLogProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyFlowLogProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy flow log proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogProposal {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub description: String,
    pub threshold: f64,
    pub quantity: i64,
    pub discount: f64,
    pub status: LegacyFlowLogProposalStatus,
}

impl LegacyFlowLogProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyFlowLogProposalStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyFlowLogProposalStatus) -> bool {
        match self.status {
            LegacyFlowLogProposalStatus::Requested => {
                return next == LegacyFlowLogProposalStatus::Revoked || next == LegacyFlowLogProposalStatus::Expired;
            }
            LegacyFlowLogProposalStatus::Granted => {
                return next == LegacyFlowLogProposalStatus::Revoked || next == LegacyFlowLogProposalStatus::Expired;
            }
            LegacyFlowLogProposalStatus::Revoked => {
                return next == LegacyFlowLogProposalStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyFlowLogProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy flow log proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.attempts as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyFlowLogProposal {
        LegacyFlowLogProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            description: self.description.clone(),
            threshold: self.threshold,
            quantity: self.quantity,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy flow log proposal in status requested.
pub fn new_legacy_flow_log_proposal(code: &str, name: &str) -> LegacyFlowLogProposal {
    LegacyFlowLogProposal {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        description: "".to_string(),
        threshold: 0.0,
        quantity: 0,
        discount: 0.0,
        status: LegacyFlowLogProposalStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_legacy_flow_log_proposal("code-355", "name-912");
        assert_eq!(item.status, LegacyFlowLogProposalStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_legacy_flow_log_proposal("code-355", "name-912");
        if item.move_to(LegacyFlowLogProposalStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyFlowLogProposalStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_legacy_flow_log_proposal("code-355", "name-912");
        item.status = LegacyFlowLogProposalStatus::Expired;
        assert!(
            item.move_to(LegacyFlowLogProposalStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
