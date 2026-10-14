use super::*;

use acme_kit::text;

/// Where a manual silence bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualSilenceBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualSilenceBundleStatus {
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

impl std::fmt::Display for ManualSilenceBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual silence bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceBundle {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub limit: i64,
    pub approved: bool,
    pub timezone: String,
    pub issuer: String,
    pub namespace: String,
    pub principal: String,
    pub status: ManualSilenceBundleStatus,
}

impl ManualSilenceBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualSilenceBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualSilenceBundleStatus) -> bool {
        match self.status {
            ManualSilenceBundleStatus::Idle => {
                match next {
                    ManualSilenceBundleStatus::Ready | ManualSilenceBundleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSilenceBundleStatus::Warming => {
                match next {
                    ManualSilenceBundleStatus::Draining | ManualSilenceBundleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSilenceBundleStatus::Ready => {
                return next == ManualSilenceBundleStatus::Stopped;
            }
            ManualSilenceBundleStatus::Draining => {
                return next == ManualSilenceBundleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualSilenceBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual silence bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "manual silence bundle", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> ManualSilenceBundle {
        ManualSilenceBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            limit: self.limit,
            approved: self.approved,
            timezone: self.timezone.clone(),
            issuer: self.issuer.clone(),
            namespace: self.namespace.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a manual silence bundle in status idle.
pub fn new_manual_silence_bundle(code: &str, label: &str) -> ManualSilenceBundle {
    ManualSilenceBundle {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        limit: 0,
        approved: false,
        timezone: "".to_string(),
        issuer: "".to_string(),
        namespace: "".to_string(),
        principal: "".to_string(),
        status: ManualSilenceBundleStatus::Idle,
    }
}
