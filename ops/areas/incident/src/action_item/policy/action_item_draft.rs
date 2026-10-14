use super::*;

use acme_kit::text;

/// Where a action item draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemDraftStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ActionItemDraftStatus {
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

impl std::fmt::Display for ActionItemDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One action item draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemDraft {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub locale: String,
    pub floor_value: f64,
    pub reviewer: String,
    pub quota_units: i64,
    pub cost: f64,
    pub timezone: String,
    pub status: ActionItemDraftStatus,
}

impl ActionItemDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemDraftStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemDraftStatus) -> bool {
        match self.status {
            ActionItemDraftStatus::Idle => {
                return next == ActionItemDraftStatus::Ready;
            }
            ActionItemDraftStatus::Warming => {
                return next == ActionItemDraftStatus::Draining || next == ActionItemDraftStatus::Stopped;
            }
            ActionItemDraftStatus::Ready => {
                match next {
                    ActionItemDraftStatus::Draining | ActionItemDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemDraftStatus::Draining => {
                return next == ActionItemDraftStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.floor_value * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemDraft {
        ActionItemDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            locale: self.locale.clone(),
            floor_value: self.floor_value,
            reviewer: self.reviewer.clone(),
            quota_units: self.quota_units,
            cost: self.cost,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a action item draft in status idle.
pub fn new_action_item_draft(code: &str, title: &str) -> ActionItemDraft {
    ActionItemDraft {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        locale: "".to_string(),
        floor_value: 0.0,
        reviewer: "".to_string(),
        quota_units: 0,
        cost: 0.0,
        timezone: "".to_string(),
        status: ActionItemDraftStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_action_item_draft("code-683", "title-988");
        assert_eq!(item.status, ActionItemDraftStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_action_item_draft("code-683", "title-988");
        if item.move_to(ActionItemDraftStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemDraftStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_action_item_draft("code-683", "title-988");
        item.status = ActionItemDraftStatus::Stopped;
        assert!(
            item.move_to(ActionItemDraftStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
