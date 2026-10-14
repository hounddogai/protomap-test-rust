use super::*;

use acme_kit::text;

/// Where a action item cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemCycleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ActionItemCycleStatus {
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

impl std::fmt::Display for ActionItemCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item cycle as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemCycle {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub discount: f64,
    pub namespace: String,
    pub checksum: String,
    pub duration_minutes: i64,
    pub fingerprint: String,
    pub status: ActionItemCycleStatus,
}

impl ActionItemCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemCycleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemCycleStatus) -> bool {
        match self.status {
            ActionItemCycleStatus::Draft => {
                return next == ActionItemCycleStatus::Active || next == ActionItemCycleStatus::Closed;
            }
            ActionItemCycleStatus::Pending => {
                match next {
                    ActionItemCycleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemCycleStatus::Active => {
                return next == ActionItemCycleStatus::Suspended || next == ActionItemCycleStatus::Closed;
            }
            ActionItemCycleStatus::Suspended => {
                return next == ActionItemCycleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "action item cycle", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.discount - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemCycle {
        ActionItemCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            discount: self.discount,
            namespace: self.namespace.clone(),
            checksum: self.checksum.clone(),
            duration_minutes: self.duration_minutes,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a action item cycle in status draft.
pub fn new_action_item_cycle(code: &str, name: &str) -> ActionItemCycle {
    ActionItemCycle {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        discount: 0.0,
        namespace: "".to_string(),
        checksum: "".to_string(),
        duration_minutes: 0,
        fingerprint: "".to_string(),
        status: ActionItemCycleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_action_item_cycle("code-49", "name-608");
        assert_eq!(item.status, ActionItemCycleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_action_item_cycle("code-49", "name-608");
        if item.move_to(ActionItemCycleStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemCycleStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_action_item_cycle("code-49", "name-608");
        item.status = ActionItemCycleStatus::Closed;
        assert!(
            item.move_to(ActionItemCycleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
