use super::*;

use acme_kit::text;

/// Where a lenient forecast checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientForecastCheckpointStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientForecastCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientForecastCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient forecast checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastCheckpoint {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub burst: i64,
    pub visible: bool,
    pub tenant_id: String,
    pub audience: String,
    pub status: LenientForecastCheckpointStatus,
}

impl LenientForecastCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientForecastCheckpointStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientForecastCheckpointStatus) -> bool {
        match self.status {
            LenientForecastCheckpointStatus::Requested => {
                return next == LenientForecastCheckpointStatus::Granted
                    || next == LenientForecastCheckpointStatus::Revoked;
            }
            LenientForecastCheckpointStatus::Granted => {
                match next {
                    LenientForecastCheckpointStatus::Revoked | LenientForecastCheckpointStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientForecastCheckpointStatus::Revoked => {
                return next == LenientForecastCheckpointStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientForecastCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient forecast checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> LenientForecastCheckpoint {
        LenientForecastCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            burst: self.burst,
            visible: self.visible,
            tenant_id: self.tenant_id.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient forecast checkpoint in status requested.
pub fn new_lenient_forecast_checkpoint(code: &str, label: &str) -> LenientForecastCheckpoint {
    LenientForecastCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        burst: 0,
        visible: false,
        tenant_id: "".to_string(),
        audience: "".to_string(),
        status: LenientForecastCheckpointStatus::Requested,
    }
}
