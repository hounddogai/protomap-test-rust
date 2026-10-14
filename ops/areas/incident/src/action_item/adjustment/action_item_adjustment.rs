use super::*;

use acme_kit::text;

/// Where a action item adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemAdjustmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ActionItemAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ActionItemAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item adjustment as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAdjustment {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub description: String,
    pub ceiling: f64,
    pub host: String,
    pub status: ActionItemAdjustmentStatus,
}

impl ActionItemAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemAdjustmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemAdjustmentStatus) -> bool {
        match self.status {
            ActionItemAdjustmentStatus::Created => {
                return next == ActionItemAdjustmentStatus::Published;
            }
            ActionItemAdjustmentStatus::Validated => {
                return next == ActionItemAdjustmentStatus::Published;
            }
            ActionItemAdjustmentStatus::Published => {
                return next == ActionItemAdjustmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "action item adjustment", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.sequence as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemAdjustment {
        ActionItemAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            description: self.description.clone(),
            ceiling: self.ceiling,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a action item adjustment in status created.
pub fn new_action_item_adjustment(code: &str, name: &str) -> ActionItemAdjustment {
    ActionItemAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        description: "".to_string(),
        ceiling: 0.0,
        host: "".to_string(),
        status: ActionItemAdjustmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_action_item_adjustment("code-140", "name-703");
        assert_eq!(item.status, ActionItemAdjustmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_action_item_adjustment("code-140", "name-703");
        if item.move_to(ActionItemAdjustmentStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemAdjustmentStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_action_item_adjustment("code-140", "name-703");
        item.status = ActionItemAdjustmentStatus::Retired;
        assert!(
            item.move_to(ActionItemAdjustmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
