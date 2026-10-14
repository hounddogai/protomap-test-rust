use super::*;

use acme_kit::text;

/// Where a budget burn manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnManifestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BudgetBurnManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BudgetBurnManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn manifest as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnManifest {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub external_ref: String,
    pub budget: f64,
    pub category: String,
    pub status: BudgetBurnManifestStatus,
}

impl BudgetBurnManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnManifestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnManifestStatus) -> bool {
        match self.status {
            BudgetBurnManifestStatus::Proposed => {
                match next {
                    BudgetBurnManifestStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnManifestStatus::Approved => {
                return next == BudgetBurnManifestStatus::Rejected || next == BudgetBurnManifestStatus::Withdrawn;
            }
            BudgetBurnManifestStatus::Rejected => {
                return next == BudgetBurnManifestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        (self.budget * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnManifest {
        BudgetBurnManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            external_ref: self.external_ref.clone(),
            budget: self.budget,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a budget burn manifest in status proposed.
pub fn new_budget_burn_manifest(code: &str, title: &str) -> BudgetBurnManifest {
    BudgetBurnManifest {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        external_ref: "".to_string(),
        budget: 0.0,
        category: "".to_string(),
        status: BudgetBurnManifestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_budget_burn_manifest("code-367", "title-835");
        assert_eq!(item.status, BudgetBurnManifestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_budget_burn_manifest("code-367", "title-835");
        if item.move_to(BudgetBurnManifestStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnManifestStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_budget_burn_manifest("code-367", "title-835");
        item.status = BudgetBurnManifestStatus::Withdrawn;
        assert!(
            item.move_to(BudgetBurnManifestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
