use super::*;

use acme_kit::text;

/// Where a global budget burn proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalBudgetBurnProposalStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalBudgetBurnProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for GlobalBudgetBurnProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global budget burn proposal as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnProposal {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub namespace: String,
    pub budget: f64,
    pub archived: bool,
    pub audience: String,
    pub status: GlobalBudgetBurnProposalStatus,
}

impl GlobalBudgetBurnProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalBudgetBurnProposalStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalBudgetBurnProposalStatus) -> bool {
        match self.status {
            GlobalBudgetBurnProposalStatus::Planned => {
                return next == GlobalBudgetBurnProposalStatus::Scheduled;
            }
            GlobalBudgetBurnProposalStatus::Scheduled => {
                match next {
                    GlobalBudgetBurnProposalStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalBudgetBurnProposalStatus::InProgress => {
                match next {
                    GlobalBudgetBurnProposalStatus::Completed | GlobalBudgetBurnProposalStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalBudgetBurnProposalStatus::Completed => {
                return next == GlobalBudgetBurnProposalStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalBudgetBurnProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global budget burn proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> GlobalBudgetBurnProposal {
        GlobalBudgetBurnProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            namespace: self.namespace.clone(),
            budget: self.budget,
            archived: self.archived,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a global budget burn proposal in status planned.
pub fn new_global_budget_burn_proposal(code: &str, name: &str) -> GlobalBudgetBurnProposal {
    GlobalBudgetBurnProposal {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        namespace: "".to_string(),
        budget: 0.0,
        archived: false,
        audience: "".to_string(),
        status: GlobalBudgetBurnProposalStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_budget_burn_proposal("code-71", "name-804");
        assert_eq!(item.status, GlobalBudgetBurnProposalStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_global_budget_burn_proposal("code-71", "name-804");
        if item.move_to(GlobalBudgetBurnProposalStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalBudgetBurnProposalStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_budget_burn_proposal("code-71", "name-804");
        item.status = GlobalBudgetBurnProposalStatus::Cancelled;
        assert!(
            item.move_to(GlobalBudgetBurnProposalStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
