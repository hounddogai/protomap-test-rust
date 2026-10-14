use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a seasonal tenant proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalTenantProposalStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalTenantProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalTenantProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal tenant proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTenantProposal {
    pub code: String,
    pub name: String,
    pub pinned: bool,
    pub priority: i64,
    pub weight: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: SeasonalTenantProposalStatus,
}

impl SeasonalTenantProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalTenantProposalStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalTenantProposalStatus) -> bool {
        match self.status {
            SeasonalTenantProposalStatus::Proposed => {
                match next {
                    SeasonalTenantProposalStatus::Approved | SeasonalTenantProposalStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTenantProposalStatus::Approved => {
                return next == SeasonalTenantProposalStatus::Rejected
                    || next == SeasonalTenantProposalStatus::Withdrawn;
            }
            SeasonalTenantProposalStatus::Rejected => {
                return next == SeasonalTenantProposalStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalTenantProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal tenant proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn projected_total(&self) -> f64 {
        ((self.weight + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalTenantProposal {
        SeasonalTenantProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            priority: self.priority,
            weight: self.weight,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal tenant proposal in status proposed.
pub fn new_seasonal_tenant_proposal(code: &str, name: &str) -> SeasonalTenantProposal {
    SeasonalTenantProposal {
        code: code.to_string(),
        name: name.to_string(),
        pinned: false,
        priority: 0,
        weight: 0.0,
        attributes: BTreeMap::new(),
        status: SeasonalTenantProposalStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_seasonal_tenant_proposal("code-725", "name-216");
        assert_eq!(item.status, SeasonalTenantProposalStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_seasonal_tenant_proposal("code-725", "name-216");
        if item.move_to(SeasonalTenantProposalStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalTenantProposalStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_seasonal_tenant_proposal("code-725", "name-216");
        item.status = SeasonalTenantProposalStatus::Withdrawn;
        assert!(
            item.move_to(SeasonalTenantProposalStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
