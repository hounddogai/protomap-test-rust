use super::*;

use acme_kit::text;

/// Where a rolling retention reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingRetentionReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingRetentionReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingRetentionReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling retention reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRetentionReservation {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub issuer: String,
    pub owner_id: String,
    pub quota_units: i64,
    pub status: RollingRetentionReservationStatus,
}

impl RollingRetentionReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingRetentionReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingRetentionReservationStatus) -> bool {
        match self.status {
            RollingRetentionReservationStatus::Requested => {
                return next == RollingRetentionReservationStatus::Granted;
            }
            RollingRetentionReservationStatus::Granted => {
                return next == RollingRetentionReservationStatus::Revoked
                    || next == RollingRetentionReservationStatus::Expired;
            }
            RollingRetentionReservationStatus::Revoked => {
                return next == RollingRetentionReservationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingRetentionReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling retention reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling retention reservation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quota_units as f64
    }

    pub fn copy(&self) -> RollingRetentionReservation {
        RollingRetentionReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            issuer: self.issuer.clone(),
            owner_id: self.owner_id.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a rolling retention reservation in status requested.
pub fn new_rolling_retention_reservation(code: &str, label: &str) -> RollingRetentionReservation {
    RollingRetentionReservation {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        issuer: "".to_string(),
        owner_id: "".to_string(),
        quota_units: 0,
        status: RollingRetentionReservationStatus::Requested,
    }
}
