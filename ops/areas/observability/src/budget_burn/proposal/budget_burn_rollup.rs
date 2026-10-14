use super::*;

use acme_kit::text;

/// Where a budget burn rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnRollupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BudgetBurnRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BudgetBurnRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn rollup as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRollup {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub weight: f64,
    pub revision: i64,
    pub checksum: String,
    pub status: BudgetBurnRollupStatus,
}

impl BudgetBurnRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnRollupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnRollupStatus) -> bool {
        match self.status {
            BudgetBurnRollupStatus::Proposed => {
                return next == BudgetBurnRollupStatus::Withdrawn;
            }
            BudgetBurnRollupStatus::Approved => {
                return next == BudgetBurnRollupStatus::Rejected || next == BudgetBurnRollupStatus::Withdrawn;
            }
            BudgetBurnRollupStatus::Rejected => {
                match next {
                    BudgetBurnRollupStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: BudgetBurnRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.ends_at as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnRollup {
        BudgetBurnRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            weight: self.weight,
            revision: self.revision,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a budget burn rollup in status proposed.
pub fn new_budget_burn_rollup(code: &str, title: &str) -> BudgetBurnRollup {
    BudgetBurnRollup {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        weight: 0.0,
        revision: 0,
        checksum: "".to_string(),
        status: BudgetBurnRollupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_budget_burn_rollup("code-612", "title-139");
        assert_eq!(item.status, BudgetBurnRollupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_budget_burn_rollup("code-612", "title-139");
        if item.move_to(BudgetBurnRollupStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnRollupStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_budget_burn_rollup("code-612", "title-139");
        item.status = BudgetBurnRollupStatus::Withdrawn;
        assert!(
            item.move_to(BudgetBurnRollupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
