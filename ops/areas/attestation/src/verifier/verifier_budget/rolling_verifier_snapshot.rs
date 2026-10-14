use super::*;

use acme_kit::text;

/// Where a rolling verifier snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingVerifierSnapshotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollingVerifierSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollingVerifierSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling verifier snapshot as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierSnapshot {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub fingerprint: String,
    pub retries: i64,
    pub label: String,
    pub quota_units: i64,
    pub maximum: f64,
    pub grace_days: i64,
    pub status: RollingVerifierSnapshotStatus,
}

impl RollingVerifierSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingVerifierSnapshotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingVerifierSnapshotStatus) -> bool {
        match self.status {
            RollingVerifierSnapshotStatus::Created => {
                match next {
                    RollingVerifierSnapshotStatus::Validated | RollingVerifierSnapshotStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingVerifierSnapshotStatus::Validated => {
                match next {
                    RollingVerifierSnapshotStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingVerifierSnapshotStatus::Published => {
                match next {
                    RollingVerifierSnapshotStatus::Retired => {
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

    pub fn move_to(&mut self, next: RollingVerifierSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling verifier snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.grace_days as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingVerifierSnapshot {
        RollingVerifierSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            fingerprint: self.fingerprint.clone(),
            retries: self.retries,
            label: self.label.clone(),
            quota_units: self.quota_units,
            maximum: self.maximum,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling verifier snapshot in status created.
pub fn new_rolling_verifier_snapshot(code: &str, name: &str) -> RollingVerifierSnapshot {
    RollingVerifierSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        fingerprint: "".to_string(),
        retries: 0,
        label: "".to_string(),
        quota_units: 0,
        maximum: 0.0,
        grace_days: 0,
        status: RollingVerifierSnapshotStatus::Created,
    }
}
