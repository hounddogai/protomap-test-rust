use super::*;

use acme_kit::text;

/// Where a scheduled identity window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledIdentityWindowStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledIdentityWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledIdentityWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled identity window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityWindow {
    pub code: String,
    pub title: String,
    pub name: String,
    pub scope: String,
    pub principal: String,
    pub quantity: i64,
    pub status: ScheduledIdentityWindowStatus,
}

impl ScheduledIdentityWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledIdentityWindowStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledIdentityWindowStatus) -> bool {
        match self.status {
            ScheduledIdentityWindowStatus::Created => {
                match next {
                    ScheduledIdentityWindowStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledIdentityWindowStatus::Validated => {
                return next == ScheduledIdentityWindowStatus::Retired;
            }
            ScheduledIdentityWindowStatus::Published => {
                return next == ScheduledIdentityWindowStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledIdentityWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled identity window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> ScheduledIdentityWindow {
        ScheduledIdentityWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            principal: self.principal.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a scheduled identity window in status created.
pub fn new_scheduled_identity_window(code: &str, title: &str) -> ScheduledIdentityWindow {
    ScheduledIdentityWindow {
        code: code.to_string(),
        title: title.to_string(),
        name: "".to_string(),
        scope: "".to_string(),
        principal: "".to_string(),
        quantity: 0,
        status: ScheduledIdentityWindowStatus::Created,
    }
}
