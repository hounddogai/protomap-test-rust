use super::*;

use acme_kit::text;

/// Where a credential snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialSnapshotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CredentialSnapshotStatus {
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

impl std::fmt::Display for CredentialSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a credential snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSnapshot {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub revision: i64,
    pub host: String,
    pub namespace: String,
    pub timezone: String,
    pub status: CredentialSnapshotStatus,
}

impl CredentialSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialSnapshotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialSnapshotStatus) -> bool {
        match self.status {
            CredentialSnapshotStatus::Idle => {
                match next {
                    CredentialSnapshotStatus::Ready | CredentialSnapshotStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialSnapshotStatus::Warming => {
                match next {
                    CredentialSnapshotStatus::Ready | CredentialSnapshotStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialSnapshotStatus::Ready => {
                return next == CredentialSnapshotStatus::Stopped;
            }
            CredentialSnapshotStatus::Draining => {
                match next {
                    CredentialSnapshotStatus::Stopped => {
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

    pub fn move_to(&mut self, next: CredentialSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "credential snapshot", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.rate - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialSnapshot {
        CredentialSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            revision: self.revision,
            host: self.host.clone(),
            namespace: self.namespace.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a credential snapshot in status idle.
pub fn new_credential_snapshot(code: &str, label: &str) -> CredentialSnapshot {
    CredentialSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        revision: 0,
        host: "".to_string(),
        namespace: "".to_string(),
        timezone: "".to_string(),
        status: CredentialSnapshotStatus::Idle,
    }
}
