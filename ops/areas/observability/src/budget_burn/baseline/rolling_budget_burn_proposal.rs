use super::*;

use acme_kit::text;

/// Where a rolling budget burn proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingBudgetBurnProposalStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingBudgetBurnProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingBudgetBurnProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling budget burn proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnProposal {
    pub code: String,
    pub label: String,
    pub ratio: f64,
    pub segment: String,
    pub source: String,
    pub revision: i64,
    pub status: RollingBudgetBurnProposalStatus,
}

impl RollingBudgetBurnProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingBudgetBurnProposalStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingBudgetBurnProposalStatus) -> bool {
        match self.status {
            RollingBudgetBurnProposalStatus::Requested => {
                match next {
                    RollingBudgetBurnProposalStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingBudgetBurnProposalStatus::Granted => {
                return next == RollingBudgetBurnProposalStatus::Revoked;
            }
            RollingBudgetBurnProposalStatus::Revoked => {
                return next == RollingBudgetBurnProposalStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingBudgetBurnProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling budget burn proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.ratio + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingBudgetBurnProposal {
        RollingBudgetBurnProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            ratio: self.ratio,
            segment: self.segment.clone(),
            source: self.source.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rolling budget burn proposal in status requested.
pub fn new_rolling_budget_burn_proposal(code: &str, label: &str) -> RollingBudgetBurnProposal {
    RollingBudgetBurnProposal {
        code: code.to_string(),
        label: label.to_string(),
        ratio: 0.0,
        segment: "".to_string(),
        source: "".to_string(),
        revision: 0,
        status: RollingBudgetBurnProposalStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rolling_budget_burn_proposal("code-925", "label-267");
        assert_eq!(item.status, RollingBudgetBurnProposalStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_rolling_budget_burn_proposal("code-925", "label-267");
        if item.move_to(RollingBudgetBurnProposalStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingBudgetBurnProposalStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rolling_budget_burn_proposal("code-925", "label-267");
        item.status = RollingBudgetBurnProposalStatus::Expired;
        assert!(
            item.move_to(RollingBudgetBurnProposalStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
