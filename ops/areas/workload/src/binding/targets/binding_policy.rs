use super::*;

use acme_kit::text;

/// Where a binding policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingPolicyStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BindingPolicyStatus {
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

impl std::fmt::Display for BindingPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingPolicy {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub priority: i64,
    pub floor_value: f64,
    pub window_days: i64,
    pub attempts: i64,
    pub category: String,
    pub status: BindingPolicyStatus,
}

impl BindingPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingPolicyStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingPolicyStatus) -> bool {
        match self.status {
            BindingPolicyStatus::Idle => {
                return next == BindingPolicyStatus::Ready || next == BindingPolicyStatus::Stopped;
            }
            BindingPolicyStatus::Warming => {
                match next {
                    BindingPolicyStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingPolicyStatus::Ready => {
                return next == BindingPolicyStatus::Stopped;
            }
            BindingPolicyStatus::Draining => {
                match next {
                    BindingPolicyStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BindingPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.attempts as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingPolicy {
        BindingPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            priority: self.priority,
            floor_value: self.floor_value,
            window_days: self.window_days,
            attempts: self.attempts,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a binding policy in status idle.
pub fn new_binding_policy(code: &str, title: &str) -> BindingPolicy {
    BindingPolicy {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        priority: 0,
        floor_value: 0.0,
        window_days: 0,
        attempts: 0,
        category: "".to_string(),
        status: BindingPolicyStatus::Idle,
    }
}
