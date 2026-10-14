use super::*;

use acme_kit::text;

/// Where a rolling credential snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingCredentialSnapshotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingCredentialSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingCredentialSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling credential snapshot as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingCredentialSnapshot {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub priority: i64,
    pub external_ref: String,
    pub summary: String,
    pub checksum: String,
    pub visible: bool,
    pub updated_at: i64,
    pub rate: f64,
    pub segment: String,
    pub status: RollingCredentialSnapshotStatus,
}

impl RollingCredentialSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingCredentialSnapshotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingCredentialSnapshotStatus) -> bool {
        match self.status {
            RollingCredentialSnapshotStatus::Requested => {
                return next == RollingCredentialSnapshotStatus::Granted
                    || next == RollingCredentialSnapshotStatus::Expired;
            }
            RollingCredentialSnapshotStatus::Granted => {
                return next == RollingCredentialSnapshotStatus::Revoked;
            }
            RollingCredentialSnapshotStatus::Revoked => {
                return next == RollingCredentialSnapshotStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingCredentialSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling credential snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling credential snapshot", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.priority as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingCredentialSnapshot {
        RollingCredentialSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            priority: self.priority,
            external_ref: self.external_ref.clone(),
            summary: self.summary.clone(),
            checksum: self.checksum.clone(),
            visible: self.visible,
            updated_at: self.updated_at,
            rate: self.rate,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rolling credential snapshot in status requested.
pub fn new_rolling_credential_snapshot(code: &str, label: &str) -> RollingCredentialSnapshot {
    RollingCredentialSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        priority: 0,
        external_ref: "".to_string(),
        summary: "".to_string(),
        checksum: "".to_string(),
        visible: false,
        updated_at: 0,
        rate: 0.0,
        segment: "".to_string(),
        status: RollingCredentialSnapshotStatus::Requested,
    }
}
