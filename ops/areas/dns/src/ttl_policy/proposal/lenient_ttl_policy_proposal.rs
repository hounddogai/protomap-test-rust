use super::*;

use acme_kit::text;

/// Where a lenient ttl policy proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientTtlPolicyProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientTtlPolicyProposalStatus {
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

impl std::fmt::Display for LenientTtlPolicyProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient ttl policy proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyProposal {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub batch_size: i64,
    pub namespace: String,
    pub approved: bool,
    pub status: LenientTtlPolicyProposalStatus,
}

impl LenientTtlPolicyProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientTtlPolicyProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientTtlPolicyProposalStatus) -> bool {
        match self.status {
            LenientTtlPolicyProposalStatus::Draft => {
                match next {
                    LenientTtlPolicyProposalStatus::Pending | LenientTtlPolicyProposalStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTtlPolicyProposalStatus::Pending => {
                return next == LenientTtlPolicyProposalStatus::Closed;
            }
            LenientTtlPolicyProposalStatus::Active => {
                match next {
                    LenientTtlPolicyProposalStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTtlPolicyProposalStatus::Suspended => {
                return next == LenientTtlPolicyProposalStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientTtlPolicyProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient ttl policy proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient ttl policy proposal", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> LenientTtlPolicyProposal {
        LenientTtlPolicyProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            batch_size: self.batch_size,
            namespace: self.namespace.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a lenient ttl policy proposal in status draft.
pub fn new_lenient_ttl_policy_proposal(code: &str, label: &str) -> LenientTtlPolicyProposal {
    LenientTtlPolicyProposal {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        batch_size: 0,
        namespace: "".to_string(),
        approved: false,
        status: LenientTtlPolicyProposalStatus::Draft,
    }
}
