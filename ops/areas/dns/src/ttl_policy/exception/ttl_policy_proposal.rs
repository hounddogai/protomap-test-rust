use super::*;

use acme_kit::text;

/// Where a ttl policy proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyProposalStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TtlPolicyProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TtlPolicyProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy proposal as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyProposal {
    pub code: String,
    pub title: String,
    pub region: String,
    pub cluster: String,
    pub visible: bool,
    pub amount: f64,
    pub status: TtlPolicyProposalStatus,
}

impl TtlPolicyProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyProposalStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyProposalStatus) -> bool {
        match self.status {
            TtlPolicyProposalStatus::Created => {
                return next == TtlPolicyProposalStatus::Retired;
            }
            TtlPolicyProposalStatus::Validated => {
                return next == TtlPolicyProposalStatus::Published || next == TtlPolicyProposalStatus::Retired;
            }
            TtlPolicyProposalStatus::Published => {
                return next == TtlPolicyProposalStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "ttl policy proposal", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> TtlPolicyProposal {
        TtlPolicyProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            cluster: self.cluster.clone(),
            visible: self.visible,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a ttl policy proposal in status created.
pub fn new_ttl_policy_proposal(code: &str, title: &str) -> TtlPolicyProposal {
    TtlPolicyProposal {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        cluster: "".to_string(),
        visible: false,
        amount: 0.0,
        status: TtlPolicyProposalStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_ttl_policy_proposal("code-131", "title-553");
        assert_eq!(item.status, TtlPolicyProposalStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_ttl_policy_proposal("code-131", "title-553");
        if item.move_to(TtlPolicyProposalStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyProposalStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_ttl_policy_proposal("code-131", "title-553");
        item.status = TtlPolicyProposalStatus::Retired;
        assert!(
            item.move_to(TtlPolicyProposalStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
