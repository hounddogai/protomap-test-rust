use super::*;

use acme_kit::text;

/// Where a weighted page proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedPageProposalStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl WeightedPageProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for WeightedPageProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted page proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPageProposal {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub issuer: String,
    pub floor_value: f64,
    pub limit: i64,
    pub external_ref: String,
    pub cost: f64,
    pub rotation_days: i64,
    pub scope: String,
    pub status: WeightedPageProposalStatus,
}

impl WeightedPageProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedPageProposalStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedPageProposalStatus) -> bool {
        match self.status {
            WeightedPageProposalStatus::Requested => {
                return next == WeightedPageProposalStatus::Granted;
            }
            WeightedPageProposalStatus::Granted => {
                match next {
                    WeightedPageProposalStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedPageProposalStatus::Revoked => {
                return next == WeightedPageProposalStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedPageProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted page proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.cost - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedPageProposal {
        WeightedPageProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            issuer: self.issuer.clone(),
            floor_value: self.floor_value,
            limit: self.limit,
            external_ref: self.external_ref.clone(),
            cost: self.cost,
            rotation_days: self.rotation_days,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a weighted page proposal in status requested.
pub fn new_weighted_page_proposal(code: &str, name: &str) -> WeightedPageProposal {
    WeightedPageProposal {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        issuer: "".to_string(),
        floor_value: 0.0,
        limit: 0,
        external_ref: "".to_string(),
        cost: 0.0,
        rotation_days: 0,
        scope: "".to_string(),
        status: WeightedPageProposalStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_weighted_page_proposal("code-485", "name-547");
        assert_eq!(item.status, WeightedPageProposalStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_weighted_page_proposal("code-485", "name-547");
        if item.move_to(WeightedPageProposalStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedPageProposalStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_weighted_page_proposal("code-485", "name-547");
        item.status = WeightedPageProposalStatus::Expired;
        assert!(
            item.move_to(WeightedPageProposalStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
