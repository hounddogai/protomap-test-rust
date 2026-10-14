use super::*;

use acme_kit::text;

/// Where a capped freeze proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedFreezeProposalStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedFreezeProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedFreezeProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped freeze proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedFreezeProposal {
    pub code: String,
    pub name: String,
    pub label: String,
    pub grace_days: i64,
    pub description: String,
    pub unit: String,
    pub host: String,
    pub retries: i64,
    pub status: CappedFreezeProposalStatus,
}

impl CappedFreezeProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedFreezeProposalStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedFreezeProposalStatus) -> bool {
        match self.status {
            CappedFreezeProposalStatus::Requested => {
                match next {
                    CappedFreezeProposalStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedFreezeProposalStatus::Granted => {
                match next {
                    CappedFreezeProposalStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedFreezeProposalStatus::Revoked => {
                match next {
                    CappedFreezeProposalStatus::Expired => {
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

    pub fn move_to(&mut self, next: CappedFreezeProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped freeze proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "capped freeze proposal", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.retries as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedFreezeProposal {
        CappedFreezeProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            description: self.description.clone(),
            unit: self.unit.clone(),
            host: self.host.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a capped freeze proposal in status requested.
pub fn new_capped_freeze_proposal(code: &str, name: &str) -> CappedFreezeProposal {
    CappedFreezeProposal {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        grace_days: 0,
        description: "".to_string(),
        unit: "".to_string(),
        host: "".to_string(),
        retries: 0,
        status: CappedFreezeProposalStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_capped_freeze_proposal("code-292", "name-450");
        assert_eq!(item.status, CappedFreezeProposalStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_capped_freeze_proposal("code-292", "name-450");
        if item.move_to(CappedFreezeProposalStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedFreezeProposalStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_capped_freeze_proposal("code-292", "name-450");
        item.status = CappedFreezeProposalStatus::Expired;
        assert!(
            item.move_to(CappedFreezeProposalStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
