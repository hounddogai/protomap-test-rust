use super::*;

use acme_kit::text;

/// Where a action item window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemWindowStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ActionItemWindowStatus {
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

impl std::fmt::Display for ActionItemWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One action item window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemWindow {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub minimum: f64,
    pub rotation_days: i64,
    pub expires_at: i64,
    pub status: ActionItemWindowStatus,
}

impl ActionItemWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemWindowStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemWindowStatus) -> bool {
        match self.status {
            ActionItemWindowStatus::Idle => {
                match next {
                    ActionItemWindowStatus::Warming | ActionItemWindowStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemWindowStatus::Warming => {
                match next {
                    ActionItemWindowStatus::Ready | ActionItemWindowStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemWindowStatus::Ready => {
                return next == ActionItemWindowStatus::Draining || next == ActionItemWindowStatus::Stopped;
            }
            ActionItemWindowStatus::Draining => {
                return next == ActionItemWindowStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item window {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "action item window", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.expires_at as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemWindow {
        ActionItemWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            minimum: self.minimum,
            rotation_days: self.rotation_days,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a action item window in status idle.
pub fn new_action_item_window(code: &str, label: &str) -> ActionItemWindow {
    ActionItemWindow {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        minimum: 0.0,
        rotation_days: 0,
        expires_at: 0,
        status: ActionItemWindowStatus::Idle,
    }
}
