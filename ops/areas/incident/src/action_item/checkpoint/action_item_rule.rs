use super::*;

use acme_kit::text;

/// Where a action item rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ActionItemRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for ActionItemRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One action item rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRule {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub discount: f64,
    pub scope: String,
    pub pinned: bool,
    pub retries: i64,
    pub quota_units: i64,
    pub summary: String,
    pub status: ActionItemRuleStatus,
}

impl ActionItemRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemRuleStatus) -> bool {
        match self.status {
            ActionItemRuleStatus::Planned => {
                match next {
                    ActionItemRuleStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemRuleStatus::Scheduled => {
                return next == ActionItemRuleStatus::Completed || next == ActionItemRuleStatus::Cancelled;
            }
            ActionItemRuleStatus::InProgress => {
                return next == ActionItemRuleStatus::Completed || next == ActionItemRuleStatus::Cancelled;
            }
            ActionItemRuleStatus::Completed => {
                match next {
                    ActionItemRuleStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ActionItemRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "action item rule", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        (self.discount * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemRule {
        ActionItemRule {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            discount: self.discount,
            scope: self.scope.clone(),
            pinned: self.pinned,
            retries: self.retries,
            quota_units: self.quota_units,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a action item rule in status planned.
pub fn new_action_item_rule(code: &str, title: &str) -> ActionItemRule {
    ActionItemRule {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        discount: 0.0,
        scope: "".to_string(),
        pinned: false,
        retries: 0,
        quota_units: 0,
        summary: "".to_string(),
        status: ActionItemRuleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_action_item_rule("code-181", "title-425");
        assert_eq!(item.status, ActionItemRuleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_action_item_rule("code-181", "title-425");
        if item.move_to(ActionItemRuleStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemRuleStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_action_item_rule("code-181", "title-425");
        item.status = ActionItemRuleStatus::Cancelled;
        assert!(
            item.move_to(ActionItemRuleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
