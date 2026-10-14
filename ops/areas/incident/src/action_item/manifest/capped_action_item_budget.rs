use super::*;

use acme_kit::text;

/// Where a capped action item budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedActionItemBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CappedActionItemBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CappedActionItemBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped action item budget as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedActionItemBudget {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub floor_value: f64,
    pub revision: i64,
    pub cipher: String,
    pub status: CappedActionItemBudgetStatus,
}

impl CappedActionItemBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedActionItemBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedActionItemBudgetStatus) -> bool {
        match self.status {
            CappedActionItemBudgetStatus::Created => {
                return next == CappedActionItemBudgetStatus::Validated;
            }
            CappedActionItemBudgetStatus::Validated => {
                match next {
                    CappedActionItemBudgetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedActionItemBudgetStatus::Published => {
                match next {
                    CappedActionItemBudgetStatus::Retired => {
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

    pub fn move_to(&mut self, next: CappedActionItemBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped action item budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped action item budget", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.floor_value - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedActionItemBudget {
        CappedActionItemBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            floor_value: self.floor_value,
            revision: self.revision,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped action item budget in status created.
pub fn new_capped_action_item_budget(code: &str, name: &str) -> CappedActionItemBudget {
    CappedActionItemBudget {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        floor_value: 0.0,
        revision: 0,
        cipher: "".to_string(),
        status: CappedActionItemBudgetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_capped_action_item_budget("code-724", "name-334");
        assert_eq!(item.status, CappedActionItemBudgetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_capped_action_item_budget("code-724", "name-334");
        if item.move_to(CappedActionItemBudgetStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedActionItemBudgetStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_capped_action_item_budget("code-724", "name-334");
        item.status = CappedActionItemBudgetStatus::Retired;
        assert!(
            item.move_to(CappedActionItemBudgetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
