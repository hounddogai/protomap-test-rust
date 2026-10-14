use super::*;

use acme_kit::text;

/// Where a binding bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BindingBundleStatus {
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

impl std::fmt::Display for BindingBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a binding bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingBundle {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub approved: bool,
    pub version: i64,
    pub status: BindingBundleStatus,
}

impl BindingBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingBundleStatus) -> bool {
        match self.status {
            BindingBundleStatus::Idle => {
                return next == BindingBundleStatus::Stopped;
            }
            BindingBundleStatus::Warming => {
                return next == BindingBundleStatus::Stopped;
            }
            BindingBundleStatus::Ready => {
                match next {
                    BindingBundleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingBundleStatus::Draining => {
                return next == BindingBundleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.version as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingBundle {
        BindingBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            approved: self.approved,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a binding bundle in status idle.
pub fn new_binding_bundle(code: &str, title: &str) -> BindingBundle {
    BindingBundle {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        approved: false,
        version: 0,
        status: BindingBundleStatus::Idle,
    }
}
