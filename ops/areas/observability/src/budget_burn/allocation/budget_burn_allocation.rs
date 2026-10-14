use super::*;

use acme_kit::text;

/// Where a budget burn allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnAllocationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BudgetBurnAllocationStatus {
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

impl std::fmt::Display for BudgetBurnAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnAllocation {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub active: bool,
    pub cluster: String,
    pub grace_days: i64,
    pub threshold: f64,
    pub approved: bool,
    pub weight: f64,
    pub margin: f64,
    pub owner_id: String,
    pub status: BudgetBurnAllocationStatus,
}

impl BudgetBurnAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnAllocationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnAllocationStatus) -> bool {
        match self.status {
            BudgetBurnAllocationStatus::Draft => {
                return next == BudgetBurnAllocationStatus::Closed;
            }
            BudgetBurnAllocationStatus::Pending => {
                match next {
                    BudgetBurnAllocationStatus::Active | BudgetBurnAllocationStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnAllocationStatus::Active => {
                return next == BudgetBurnAllocationStatus::Closed;
            }
            BudgetBurnAllocationStatus::Suspended => {
                return next == BudgetBurnAllocationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "budget burn allocation", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.threshold + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnAllocation {
        BudgetBurnAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            active: self.active,
            cluster: self.cluster.clone(),
            grace_days: self.grace_days,
            threshold: self.threshold,
            approved: self.approved,
            weight: self.weight,
            margin: self.margin,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a budget burn allocation in status draft.
pub fn new_budget_burn_allocation(code: &str, name: &str) -> BudgetBurnAllocation {
    BudgetBurnAllocation {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        active: false,
        cluster: "".to_string(),
        grace_days: 0,
        threshold: 0.0,
        approved: false,
        weight: 0.0,
        margin: 0.0,
        owner_id: "".to_string(),
        status: BudgetBurnAllocationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_budget_burn_allocation("code-879", "name-157");
        assert_eq!(item.status, BudgetBurnAllocationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_budget_burn_allocation("code-879", "name-157");
        if item.move_to(BudgetBurnAllocationStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnAllocationStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_budget_burn_allocation("code-879", "name-157");
        item.status = BudgetBurnAllocationStatus::Closed;
        assert!(
            item.move_to(BudgetBurnAllocationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
