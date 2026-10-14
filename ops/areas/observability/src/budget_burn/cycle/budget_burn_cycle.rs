use super::*;

use acme_kit::text;

/// Where a budget burn cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BudgetBurnCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BudgetBurnCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnCycle {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub checksum: String,
    pub batch_size: i64,
    pub capacity: i64,
    pub minimum: f64,
    pub active: bool,
    pub currency: String,
    pub quota_units: i64,
    pub status: BudgetBurnCycleStatus,
}

impl BudgetBurnCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnCycleStatus) -> bool {
        match self.status {
            BudgetBurnCycleStatus::Proposed => {
                match next {
                    BudgetBurnCycleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnCycleStatus::Approved => {
                match next {
                    BudgetBurnCycleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnCycleStatus::Rejected => {
                return next == BudgetBurnCycleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "budget burn cycle", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.batch_size as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnCycle {
        BudgetBurnCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            checksum: self.checksum.clone(),
            batch_size: self.batch_size,
            capacity: self.capacity,
            minimum: self.minimum,
            active: self.active,
            currency: self.currency.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a budget burn cycle in status proposed.
pub fn new_budget_burn_cycle(code: &str, title: &str) -> BudgetBurnCycle {
    BudgetBurnCycle {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        checksum: "".to_string(),
        batch_size: 0,
        capacity: 0,
        minimum: 0.0,
        active: false,
        currency: "".to_string(),
        quota_units: 0,
        status: BudgetBurnCycleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_budget_burn_cycle("code-228", "title-416");
        assert_eq!(item.status, BudgetBurnCycleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_budget_burn_cycle("code-228", "title-416");
        if item.move_to(BudgetBurnCycleStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnCycleStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_budget_burn_cycle("code-228", "title-416");
        item.status = BudgetBurnCycleStatus::Withdrawn;
        assert!(
            item.move_to(BudgetBurnCycleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
