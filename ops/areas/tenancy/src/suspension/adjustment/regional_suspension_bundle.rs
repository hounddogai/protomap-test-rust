use super::*;

use acme_kit::text;

/// Where a regional suspension bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalSuspensionBundleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalSuspensionBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalSuspensionBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional suspension bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionBundle {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub fingerprint: String,
    pub checksum: String,
    pub cluster: String,
    pub phone: String,
    pub ends_at: i64,
    pub status: RegionalSuspensionBundleStatus,
}

impl RegionalSuspensionBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalSuspensionBundleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalSuspensionBundleStatus) -> bool {
        match self.status {
            RegionalSuspensionBundleStatus::Proposed => {
                return next == RegionalSuspensionBundleStatus::Approved
                    || next == RegionalSuspensionBundleStatus::Withdrawn;
            }
            RegionalSuspensionBundleStatus::Approved => {
                return next == RegionalSuspensionBundleStatus::Rejected
                    || next == RegionalSuspensionBundleStatus::Withdrawn;
            }
            RegionalSuspensionBundleStatus::Rejected => {
                return next == RegionalSuspensionBundleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalSuspensionBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional suspension bundle {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional suspension bundle", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.rotation_days as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalSuspensionBundle {
        RegionalSuspensionBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            fingerprint: self.fingerprint.clone(),
            checksum: self.checksum.clone(),
            cluster: self.cluster.clone(),
            phone: self.phone.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a regional suspension bundle in status proposed.
pub fn new_regional_suspension_bundle(code: &str, label: &str) -> RegionalSuspensionBundle {
    RegionalSuspensionBundle {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        fingerprint: "".to_string(),
        checksum: "".to_string(),
        cluster: "".to_string(),
        phone: "".to_string(),
        ends_at: 0,
        status: RegionalSuspensionBundleStatus::Proposed,
    }
}
