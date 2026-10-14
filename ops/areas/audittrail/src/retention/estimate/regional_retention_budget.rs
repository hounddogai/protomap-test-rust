use super::*;

use acme_kit::text;

/// Where a regional retention budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRetentionBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalRetentionBudgetStatus {
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

impl std::fmt::Display for RegionalRetentionBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional retention budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionBudget {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub created_at: i64,
    pub cipher: String,
    pub status: RegionalRetentionBudgetStatus,
}

impl RegionalRetentionBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRetentionBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRetentionBudgetStatus) -> bool {
        match self.status {
            RegionalRetentionBudgetStatus::Draft => {
                match next {
                    RegionalRetentionBudgetStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRetentionBudgetStatus::Pending => {
                return next == RegionalRetentionBudgetStatus::Suspended
                    || next == RegionalRetentionBudgetStatus::Closed;
            }
            RegionalRetentionBudgetStatus::Active => {
                return next == RegionalRetentionBudgetStatus::Suspended
                    || next == RegionalRetentionBudgetStatus::Closed;
            }
            RegionalRetentionBudgetStatus::Suspended => {
                return next == RegionalRetentionBudgetStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalRetentionBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional retention budget {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "regional retention budget", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.created_at as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRetentionBudget {
        RegionalRetentionBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            created_at: self.created_at,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional retention budget in status draft.
pub fn new_regional_retention_budget(code: &str, name: &str) -> RegionalRetentionBudget {
    RegionalRetentionBudget {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        created_at: 0,
        cipher: "".to_string(),
        status: RegionalRetentionBudgetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_regional_retention_budget("code-4", "name-224");
        assert_eq!(item.status, RegionalRetentionBudgetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_regional_retention_budget("code-4", "name-224");
        if item.move_to(RegionalRetentionBudgetStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalRetentionBudgetStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_regional_retention_budget("code-4", "name-224");
        item.status = RegionalRetentionBudgetStatus::Closed;
        assert!(
            item.move_to(RegionalRetentionBudgetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
