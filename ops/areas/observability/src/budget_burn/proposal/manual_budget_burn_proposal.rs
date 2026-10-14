use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a manual budget burn proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualBudgetBurnProposalStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualBudgetBurnProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualBudgetBurnProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual budget burn proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnProposal {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub minimum: f64,
    pub threshold: f64,
    pub category: String,
    pub visible: bool,
    pub active: bool,
    pub attributes: BTreeMap<String, String>,
    pub segment: String,
    pub country: String,
    pub status: ManualBudgetBurnProposalStatus,
}

impl ManualBudgetBurnProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualBudgetBurnProposalStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualBudgetBurnProposalStatus) -> bool {
        match self.status {
            ManualBudgetBurnProposalStatus::Requested => {
                return next == ManualBudgetBurnProposalStatus::Granted;
            }
            ManualBudgetBurnProposalStatus::Granted => {
                return next == ManualBudgetBurnProposalStatus::Revoked
                    || next == ManualBudgetBurnProposalStatus::Expired;
            }
            ManualBudgetBurnProposalStatus::Revoked => {
                return next == ManualBudgetBurnProposalStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualBudgetBurnProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual budget burn proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual budget burn proposal", self.code)
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

    pub fn weighted_exposure(&self) -> f64 {
        ((self.window_days as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualBudgetBurnProposal {
        ManualBudgetBurnProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            minimum: self.minimum,
            threshold: self.threshold,
            category: self.category.clone(),
            visible: self.visible,
            active: self.active,
            attributes: self.attributes.clone(),
            segment: self.segment.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a manual budget burn proposal in status requested.
pub fn new_manual_budget_burn_proposal(code: &str, title: &str) -> ManualBudgetBurnProposal {
    ManualBudgetBurnProposal {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        minimum: 0.0,
        threshold: 0.0,
        category: "".to_string(),
        visible: false,
        active: false,
        attributes: BTreeMap::new(),
        segment: "".to_string(),
        country: "".to_string(),
        status: ManualBudgetBurnProposalStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_budget_burn_proposal("code-658", "title-824");
        assert_eq!(item.status, ManualBudgetBurnProposalStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_manual_budget_burn_proposal("code-658", "title-824");
        if item.move_to(ManualBudgetBurnProposalStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualBudgetBurnProposalStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_budget_burn_proposal("code-658", "title-824");
        item.status = ManualBudgetBurnProposalStatus::Expired;
        assert!(
            item.move_to(ManualBudgetBurnProposalStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
