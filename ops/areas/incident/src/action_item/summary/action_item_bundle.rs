use super::*;

use acme_kit::text;

/// Where a action item bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemBundleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ActionItemBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ActionItemBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a action item bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBundle {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub label: String,
    pub fingerprint: String,
    pub status: ActionItemBundleStatus,
}

impl ActionItemBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemBundleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemBundleStatus) -> bool {
        match self.status {
            ActionItemBundleStatus::New => {
                return next == ActionItemBundleStatus::Running || next == ActionItemBundleStatus::Done;
            }
            ActionItemBundleStatus::Queued => {
                return next == ActionItemBundleStatus::Running || next == ActionItemBundleStatus::Done;
            }
            ActionItemBundleStatus::Running => {
                return next == ActionItemBundleStatus::Done;
            }
            ActionItemBundleStatus::Done => {
                match next {
                    ActionItemBundleStatus::Failed => {
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

    pub fn move_to(&mut self, next: ActionItemBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> ActionItemBundle {
        ActionItemBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a action item bundle in status new.
pub fn new_action_item_bundle(code: &str, name: &str) -> ActionItemBundle {
    ActionItemBundle {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        label: "".to_string(),
        fingerprint: "".to_string(),
        status: ActionItemBundleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_action_item_bundle("code-231", "name-467");
        assert_eq!(item.status, ActionItemBundleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_action_item_bundle("code-231", "name-467");
        if item.move_to(ActionItemBundleStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemBundleStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_action_item_bundle("code-231", "name-467");
        item.status = ActionItemBundleStatus::Failed;
        assert!(
            item.move_to(ActionItemBundleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
