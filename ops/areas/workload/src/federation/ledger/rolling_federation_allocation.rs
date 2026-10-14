use super::*;

use acme_kit::text;

/// Where a rolling federation allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingFederationAllocationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingFederationAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingFederationAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling federation allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationAllocation {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub quantity: i64,
    pub port: i64,
    pub fingerprint: String,
    pub floor_value: f64,
    pub status: RollingFederationAllocationStatus,
}

impl RollingFederationAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingFederationAllocationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingFederationAllocationStatus) -> bool {
        match self.status {
            RollingFederationAllocationStatus::Requested => {
                match next {
                    RollingFederationAllocationStatus::Granted | RollingFederationAllocationStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingFederationAllocationStatus::Granted => {
                return next == RollingFederationAllocationStatus::Expired;
            }
            RollingFederationAllocationStatus::Revoked => {
                return next == RollingFederationAllocationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingFederationAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling federation allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling federation allocation", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.floor_value - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingFederationAllocation {
        RollingFederationAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            quantity: self.quantity,
            port: self.port,
            fingerprint: self.fingerprint.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rolling federation allocation in status requested.
pub fn new_rolling_federation_allocation(code: &str, label: &str) -> RollingFederationAllocation {
    RollingFederationAllocation {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        quantity: 0,
        port: 0,
        fingerprint: "".to_string(),
        floor_value: 0.0,
        status: RollingFederationAllocationStatus::Requested,
    }
}
