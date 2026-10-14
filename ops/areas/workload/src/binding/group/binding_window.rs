use super::*;

use acme_kit::text;

/// Where a binding window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingWindowStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BindingWindowStatus {
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

impl std::fmt::Display for BindingWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingWindow {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub contact_email: String,
    pub grace_days: i64,
    pub status: BindingWindowStatus,
}

impl BindingWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingWindowStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingWindowStatus) -> bool {
        match self.status {
            BindingWindowStatus::Idle => {
                return next == BindingWindowStatus::Warming || next == BindingWindowStatus::Ready;
            }
            BindingWindowStatus::Warming => {
                return next == BindingWindowStatus::Ready || next == BindingWindowStatus::Stopped;
            }
            BindingWindowStatus::Ready => {
                return next == BindingWindowStatus::Stopped;
            }
            BindingWindowStatus::Draining => {
                match next {
                    BindingWindowStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BindingWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding window {} cannot move from {} to {}",
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
        self.grace_days as f64
    }

    pub fn copy(&self) -> BindingWindow {
        BindingWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            principal: self.principal.clone(),
            contact_email: self.contact_email.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a binding window in status idle.
pub fn new_binding_window(code: &str, title: &str) -> BindingWindow {
    BindingWindow {
        code: code.to_string(),
        title: title.to_string(),
        principal: "".to_string(),
        contact_email: "".to_string(),
        grace_days: 0,
        status: BindingWindowStatus::Idle,
    }
}
