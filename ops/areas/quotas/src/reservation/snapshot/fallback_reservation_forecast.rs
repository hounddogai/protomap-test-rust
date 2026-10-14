use super::*;

use acme_kit::text;

/// Where a fallback reservation forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackReservationForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackReservationForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackReservationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback reservation forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationForecast {
    pub code: String,
    pub title: String,
    pub unit: String,
    pub cipher: String,
    pub created_at: i64,
    pub capacity: i64,
    pub audience: String,
    pub window_days: i64,
    pub cost: f64,
    pub status: FallbackReservationForecastStatus,
}

impl FallbackReservationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackReservationForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackReservationForecastStatus) -> bool {
        match self.status {
            FallbackReservationForecastStatus::Requested => {
                return next == FallbackReservationForecastStatus::Expired;
            }
            FallbackReservationForecastStatus::Granted => {
                match next {
                    FallbackReservationForecastStatus::Revoked | FallbackReservationForecastStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackReservationForecastStatus::Revoked => {
                match next {
                    FallbackReservationForecastStatus::Expired => {
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

    pub fn move_to(&mut self, next: FallbackReservationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback reservation forecast {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.capacity as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackReservationForecast {
        FallbackReservationForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            cipher: self.cipher.clone(),
            created_at: self.created_at,
            capacity: self.capacity,
            audience: self.audience.clone(),
            window_days: self.window_days,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a fallback reservation forecast in status requested.
pub fn new_fallback_reservation_forecast(code: &str, title: &str) -> FallbackReservationForecast {
    FallbackReservationForecast {
        code: code.to_string(),
        title: title.to_string(),
        unit: "".to_string(),
        cipher: "".to_string(),
        created_at: 0,
        capacity: 0,
        audience: "".to_string(),
        window_days: 0,
        cost: 0.0,
        status: FallbackReservationForecastStatus::Requested,
    }
}
