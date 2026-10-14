use super::*;

use acme_kit::text;

/// Where a fallback silence reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackSilenceReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackSilenceReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackSilenceReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback silence reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSilenceReservation {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub minimum: f64,
    pub grace_days: i64,
    pub principal: String,
    pub created_at: i64,
    pub budget: f64,
    pub weight: f64,
    pub threshold: f64,
    pub rotation_days: i64,
    pub status: FallbackSilenceReservationStatus,
}

impl FallbackSilenceReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackSilenceReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackSilenceReservationStatus) -> bool {
        match self.status {
            FallbackSilenceReservationStatus::Requested => {
                return next == FallbackSilenceReservationStatus::Granted;
            }
            FallbackSilenceReservationStatus::Granted => {
                match next {
                    FallbackSilenceReservationStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackSilenceReservationStatus::Revoked => {
                match next {
                    FallbackSilenceReservationStatus::Expired => {
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

    pub fn move_to(&mut self, next: FallbackSilenceReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback silence reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.budget + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackSilenceReservation {
        FallbackSilenceReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            minimum: self.minimum,
            grace_days: self.grace_days,
            principal: self.principal.clone(),
            created_at: self.created_at,
            budget: self.budget,
            weight: self.weight,
            threshold: self.threshold,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a fallback silence reservation in status requested.
pub fn new_fallback_silence_reservation(code: &str, title: &str) -> FallbackSilenceReservation {
    FallbackSilenceReservation {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        minimum: 0.0,
        grace_days: 0,
        principal: "".to_string(),
        created_at: 0,
        budget: 0.0,
        weight: 0.0,
        threshold: 0.0,
        rotation_days: 0,
        status: FallbackSilenceReservationStatus::Requested,
    }
}
