use super::*;

use acme_kit::text;

/// Where a action item budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemBudgetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ActionItemBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ActionItemBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a action item budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBudget {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub pinned: bool,
    pub quota_units: i64,
    pub amount: f64,
    pub updated_at: i64,
    pub status: ActionItemBudgetStatus,
}

impl ActionItemBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemBudgetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemBudgetStatus) -> bool {
        match self.status {
            ActionItemBudgetStatus::Idle => {
                return next == ActionItemBudgetStatus::Draining || next == ActionItemBudgetStatus::Stopped;
            }
            ActionItemBudgetStatus::Warming => {
                match next {
                    ActionItemBudgetStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemBudgetStatus::Ready => {
                return next == ActionItemBudgetStatus::Stopped;
            }
            ActionItemBudgetStatus::Draining => {
                match next {
                    ActionItemBudgetStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ActionItemBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item budget {} cannot move from {} to {}",
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
        format!("{} {}", "action item budget", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.expires_at as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemBudget {
        ActionItemBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            pinned: self.pinned,
            quota_units: self.quota_units,
            amount: self.amount,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a action item budget in status idle.
pub fn new_action_item_budget(code: &str, name: &str) -> ActionItemBudget {
    ActionItemBudget {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        pinned: false,
        quota_units: 0,
        amount: 0.0,
        updated_at: 0,
        status: ActionItemBudgetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_action_item_budget("code-541", "name-925");
        assert_eq!(item.status, ActionItemBudgetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_action_item_budget("code-541", "name-925");
        if item.move_to(ActionItemBudgetStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemBudgetStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_action_item_budget("code-541", "name-925");
        item.status = ActionItemBudgetStatus::Stopped;
        assert!(
            item.move_to(ActionItemBudgetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
