use super::*;

use acme_kit::text;

/// Where a action item override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemOverrideStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ActionItemOverrideStatus {
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

impl std::fmt::Display for ActionItemOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a action item override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemOverride {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub discount: f64,
    pub ends_at: i64,
    pub archived: bool,
    pub batch_size: i64,
    pub phone: String,
    pub active: bool,
    pub status: ActionItemOverrideStatus,
}

impl ActionItemOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemOverrideStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemOverrideStatus) -> bool {
        match self.status {
            ActionItemOverrideStatus::New => {
                return next == ActionItemOverrideStatus::Queued;
            }
            ActionItemOverrideStatus::Queued => {
                match next {
                    ActionItemOverrideStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemOverrideStatus::Running => {
                match next {
                    ActionItemOverrideStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemOverrideStatus::Done => {
                match next {
                    ActionItemOverrideStatus::Failed => {
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

    pub fn move_to(&mut self, next: ActionItemOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "action item override", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.ceiling + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemOverride {
        ActionItemOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            discount: self.discount,
            ends_at: self.ends_at,
            archived: self.archived,
            batch_size: self.batch_size,
            phone: self.phone.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a action item override in status new.
pub fn new_action_item_override(code: &str, name: &str) -> ActionItemOverride {
    ActionItemOverride {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        discount: 0.0,
        ends_at: 0,
        archived: false,
        batch_size: 0,
        phone: "".to_string(),
        active: false,
        status: ActionItemOverrideStatus::New,
    }
}
