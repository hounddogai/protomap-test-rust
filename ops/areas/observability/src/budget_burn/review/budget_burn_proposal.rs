use super::*;

use acme_kit::text;

/// Where a budget burn proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BudgetBurnProposalStatus {
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

impl std::fmt::Display for BudgetBurnProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnProposal {
    pub code: String,
    pub name: String,
    pub source: String,
    pub reviewer: String,
    pub amount: f64,
    pub status: BudgetBurnProposalStatus,
}

impl BudgetBurnProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnProposalStatus) -> bool {
        match self.status {
            BudgetBurnProposalStatus::Draft => {
                match next {
                    BudgetBurnProposalStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnProposalStatus::Pending => {
                match next {
                    BudgetBurnProposalStatus::Suspended | BudgetBurnProposalStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnProposalStatus::Active => {
                match next {
                    BudgetBurnProposalStatus::Suspended | BudgetBurnProposalStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnProposalStatus::Suspended => {
                return next == BudgetBurnProposalStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn proposal {} cannot move from {} to {}",
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
        self.amount
    }

    pub fn copy(&self) -> BudgetBurnProposal {
        BudgetBurnProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            reviewer: self.reviewer.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a budget burn proposal in status draft.
pub fn new_budget_burn_proposal(code: &str, name: &str) -> BudgetBurnProposal {
    BudgetBurnProposal {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        reviewer: "".to_string(),
        amount: 0.0,
        status: BudgetBurnProposalStatus::Draft,
    }
}
