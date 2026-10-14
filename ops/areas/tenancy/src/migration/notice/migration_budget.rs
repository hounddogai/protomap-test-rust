use super::*;

use acme_kit::text;

/// Where a migration budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationBudgetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl MigrationBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for MigrationBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One migration budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBudget {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub audience: String,
    pub expires_at: i64,
    pub amount: f64,
    pub status: MigrationBudgetStatus,
}

impl MigrationBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationBudgetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationBudgetStatus) -> bool {
        match self.status {
            MigrationBudgetStatus::Proposed => {
                return next == MigrationBudgetStatus::Approved || next == MigrationBudgetStatus::Withdrawn;
            }
            MigrationBudgetStatus::Approved => {
                return next == MigrationBudgetStatus::Rejected || next == MigrationBudgetStatus::Withdrawn;
            }
            MigrationBudgetStatus::Rejected => {
                match next {
                    MigrationBudgetStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: MigrationBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "migration budget", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.amount + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationBudget {
        MigrationBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            audience: self.audience.clone(),
            expires_at: self.expires_at,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a migration budget in status proposed.
pub fn new_migration_budget(code: &str, name: &str) -> MigrationBudget {
    MigrationBudget {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        audience: "".to_string(),
        expires_at: 0,
        amount: 0.0,
        status: MigrationBudgetStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_migration_budget("code-750", "name-724");
        assert_eq!(item.status, MigrationBudgetStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_migration_budget("code-750", "name-724");
        if item.move_to(MigrationBudgetStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationBudgetStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_migration_budget("code-750", "name-724");
        item.status = MigrationBudgetStatus::Withdrawn;
        assert!(
            item.move_to(MigrationBudgetStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
