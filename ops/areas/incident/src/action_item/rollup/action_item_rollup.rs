use super::*;

use acme_kit::text;

/// Where a action item rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemRollupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ActionItemRollupStatus {
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

impl std::fmt::Display for ActionItemRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One action item rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRollup {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub revision: i64,
    pub key_id: String,
    pub locale: String,
    pub budget: f64,
    pub active: bool,
    pub timezone: String,
    pub status: ActionItemRollupStatus,
}

impl ActionItemRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemRollupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemRollupStatus) -> bool {
        match self.status {
            ActionItemRollupStatus::Idle => {
                match next {
                    ActionItemRollupStatus::Warming | ActionItemRollupStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemRollupStatus::Warming => {
                return next == ActionItemRollupStatus::Draining;
            }
            ActionItemRollupStatus::Ready => {
                return next == ActionItemRollupStatus::Draining || next == ActionItemRollupStatus::Stopped;
            }
            ActionItemRollupStatus::Draining => {
                match next {
                    ActionItemRollupStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ActionItemRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.budget - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemRollup {
        ActionItemRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            revision: self.revision,
            key_id: self.key_id.clone(),
            locale: self.locale.clone(),
            budget: self.budget,
            active: self.active,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a action item rollup in status idle.
pub fn new_action_item_rollup(code: &str, label: &str) -> ActionItemRollup {
    ActionItemRollup {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        revision: 0,
        key_id: "".to_string(),
        locale: "".to_string(),
        budget: 0.0,
        active: false,
        timezone: "".to_string(),
        status: ActionItemRollupStatus::Idle,
    }
}
