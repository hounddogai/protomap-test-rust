use super::*;

use acme_kit::text;

/// Where a adaptive ttl policy proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveTtlPolicyProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptiveTtlPolicyProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for AdaptiveTtlPolicyProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive ttl policy proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTtlPolicyProposal {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub score: f64,
    pub principal: String,
    pub revision: i64,
    pub budget: f64,
    pub status: AdaptiveTtlPolicyProposalStatus,
}

impl AdaptiveTtlPolicyProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveTtlPolicyProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveTtlPolicyProposalStatus) -> bool {
        match self.status {
            AdaptiveTtlPolicyProposalStatus::Draft => {
                return next == AdaptiveTtlPolicyProposalStatus::Suspended
                    || next == AdaptiveTtlPolicyProposalStatus::Closed;
            }
            AdaptiveTtlPolicyProposalStatus::Pending => {
                return next == AdaptiveTtlPolicyProposalStatus::Active
                    || next == AdaptiveTtlPolicyProposalStatus::Suspended;
            }
            AdaptiveTtlPolicyProposalStatus::Active => {
                match next {
                    AdaptiveTtlPolicyProposalStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveTtlPolicyProposalStatus::Suspended => {
                match next {
                    AdaptiveTtlPolicyProposalStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveTtlPolicyProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive ttl policy proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.score * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveTtlPolicyProposal {
        AdaptiveTtlPolicyProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            score: self.score,
            principal: self.principal.clone(),
            revision: self.revision,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a adaptive ttl policy proposal in status draft.
pub fn new_adaptive_ttl_policy_proposal(code: &str, label: &str) -> AdaptiveTtlPolicyProposal {
    AdaptiveTtlPolicyProposal {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        score: 0.0,
        principal: "".to_string(),
        revision: 0,
        budget: 0.0,
        status: AdaptiveTtlPolicyProposalStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_adaptive_ttl_policy_proposal("code-188", "label-413");
        assert_eq!(item.status, AdaptiveTtlPolicyProposalStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_adaptive_ttl_policy_proposal("code-188", "label-413");
        if item.move_to(AdaptiveTtlPolicyProposalStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveTtlPolicyProposalStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_adaptive_ttl_policy_proposal("code-188", "label-413");
        item.status = AdaptiveTtlPolicyProposalStatus::Closed;
        assert!(
            item.move_to(AdaptiveTtlPolicyProposalStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
