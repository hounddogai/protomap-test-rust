use super::*;

use acme_kit::text;

/// Where a budget burn bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnBundleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BudgetBurnBundleStatus {
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

impl std::fmt::Display for BudgetBurnBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBundle {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub score: f64,
    pub floor_value: f64,
    pub host: String,
    pub label: String,
    pub rate: f64,
    pub created_at: i64,
    pub status: BudgetBurnBundleStatus,
}

impl BudgetBurnBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnBundleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnBundleStatus) -> bool {
        match self.status {
            BudgetBurnBundleStatus::Draft => {
                match next {
                    BudgetBurnBundleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnBundleStatus::Pending => {
                return next == BudgetBurnBundleStatus::Active;
            }
            BudgetBurnBundleStatus::Active => {
                match next {
                    BudgetBurnBundleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnBundleStatus::Suspended => {
                return next == BudgetBurnBundleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "budget burn bundle", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.created_at as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnBundle {
        BudgetBurnBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            score: self.score,
            floor_value: self.floor_value,
            host: self.host.clone(),
            label: self.label.clone(),
            rate: self.rate,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a budget burn bundle in status draft.
pub fn new_budget_burn_bundle(code: &str, name: &str) -> BudgetBurnBundle {
    BudgetBurnBundle {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        score: 0.0,
        floor_value: 0.0,
        host: "".to_string(),
        label: "".to_string(),
        rate: 0.0,
        created_at: 0,
        status: BudgetBurnBundleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_budget_burn_bundle("code-293", "name-659");
        assert_eq!(item.status, BudgetBurnBundleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_budget_burn_bundle("code-293", "name-659");
        if item.move_to(BudgetBurnBundleStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnBundleStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_budget_burn_bundle("code-293", "name-659");
        item.status = BudgetBurnBundleStatus::Closed;
        assert!(
            item.move_to(BudgetBurnBundleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
