use super::*;

use acme_kit::text;

/// Where a action item allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemAllocationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ActionItemAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for ActionItemAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item allocation as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAllocation {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub grace_days: i64,
    pub namespace: String,
    pub visible: bool,
    pub cost: f64,
    pub status: ActionItemAllocationStatus,
}

impl ActionItemAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemAllocationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemAllocationStatus) -> bool {
        match self.status {
            ActionItemAllocationStatus::Open => {
                return next == ActionItemAllocationStatus::Archived;
            }
            ActionItemAllocationStatus::InReview => {
                match next {
                    ActionItemAllocationStatus::Reopened | ActionItemAllocationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemAllocationStatus::Resolved => {
                return next == ActionItemAllocationStatus::Reopened;
            }
            ActionItemAllocationStatus::Reopened => {
                return next == ActionItemAllocationStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "action item allocation", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.grace_days as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemAllocation {
        ActionItemAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            grace_days: self.grace_days,
            namespace: self.namespace.clone(),
            visible: self.visible,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a action item allocation in status open.
pub fn new_action_item_allocation(code: &str, title: &str) -> ActionItemAllocation {
    ActionItemAllocation {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        grace_days: 0,
        namespace: "".to_string(),
        visible: false,
        cost: 0.0,
        status: ActionItemAllocationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_action_item_allocation("code-922", "title-595");
        assert_eq!(item.status, ActionItemAllocationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_action_item_allocation("code-922", "title-595");
        if item.move_to(ActionItemAllocationStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemAllocationStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_action_item_allocation("code-922", "title-595");
        item.status = ActionItemAllocationStatus::Archived;
        assert!(
            item.move_to(ActionItemAllocationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
