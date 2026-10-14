use super::*;

use acme_kit::text;

/// Where a delegation manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DelegationManifestStatus {
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

impl std::fmt::Display for DelegationManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a delegation manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationManifest {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub quota_units: i64,
    pub locale: String,
    pub segment: String,
    pub port: i64,
    pub status: DelegationManifestStatus,
}

impl DelegationManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationManifestStatus) -> bool {
        match self.status {
            DelegationManifestStatus::Idle => {
                match next {
                    DelegationManifestStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationManifestStatus::Warming => {
                return next == DelegationManifestStatus::Stopped;
            }
            DelegationManifestStatus::Ready => {
                match next {
                    DelegationManifestStatus::Draining | DelegationManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationManifestStatus::Draining => {
                return next == DelegationManifestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "delegation manifest", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.port as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationManifest {
        DelegationManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            quota_units: self.quota_units,
            locale: self.locale.clone(),
            segment: self.segment.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a delegation manifest in status idle.
pub fn new_delegation_manifest(code: &str, name: &str) -> DelegationManifest {
    DelegationManifest {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        quota_units: 0,
        locale: "".to_string(),
        segment: "".to_string(),
        port: 0,
        status: DelegationManifestStatus::Idle,
    }
}
