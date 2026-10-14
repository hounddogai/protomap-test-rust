use super::*;

use acme_kit::text;

/// Where a policy proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PolicyProposalStatus {
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

impl std::fmt::Display for PolicyProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A policy proposal as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyProposal {
    pub code: String,
    pub title: String,
    pub country: String,
    pub segment: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub sequence: i64,
    pub version: i64,
    pub status: PolicyProposalStatus,
}

impl PolicyProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyProposalStatus) -> bool {
        match self.status {
            PolicyProposalStatus::Draft => {
                return next == PolicyProposalStatus::Active;
            }
            PolicyProposalStatus::Pending => {
                match next {
                    PolicyProposalStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyProposalStatus::Active => {
                return next == PolicyProposalStatus::Suspended;
            }
            PolicyProposalStatus::Suspended => {
                match next {
                    PolicyProposalStatus::Closed => {
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

    pub fn move_to(&mut self, next: PolicyProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "policy proposal", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.version as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyProposal {
        PolicyProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            segment: self.segment.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            sequence: self.sequence,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a policy proposal in status draft.
pub fn new_policy_proposal(code: &str, title: &str) -> PolicyProposal {
    PolicyProposal {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        segment: "".to_string(),
        name: "".to_string(),
        ttl_seconds: 0,
        sequence: 0,
        version: 0,
        status: PolicyProposalStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_policy_proposal("code-484", "title-610");
        assert_eq!(item.status, PolicyProposalStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_policy_proposal("code-484", "title-610");
        if item.move_to(PolicyProposalStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyProposalStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_policy_proposal("code-484", "title-610");
        item.status = PolicyProposalStatus::Closed;
        assert!(
            item.move_to(PolicyProposalStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
