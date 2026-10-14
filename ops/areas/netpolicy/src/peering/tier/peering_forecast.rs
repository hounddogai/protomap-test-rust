use super::*;

use acme_kit::text;

/// Where a peering forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PeeringForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PeeringForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringForecast {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub timezone: String,
    pub capacity: i64,
    pub threshold: f64,
    pub approved: bool,
    pub status: PeeringForecastStatus,
}

impl PeeringForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringForecastStatus) -> bool {
        match self.status {
            PeeringForecastStatus::Requested => {
                return next == PeeringForecastStatus::Granted || next == PeeringForecastStatus::Expired;
            }
            PeeringForecastStatus::Granted => {
                return next == PeeringForecastStatus::Revoked || next == PeeringForecastStatus::Expired;
            }
            PeeringForecastStatus::Revoked => {
                return next == PeeringForecastStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering forecast {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ends_at as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringForecast {
        PeeringForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            timezone: self.timezone.clone(),
            capacity: self.capacity,
            threshold: self.threshold,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a peering forecast in status requested.
pub fn new_peering_forecast(code: &str, title: &str) -> PeeringForecast {
    PeeringForecast {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        timezone: "".to_string(),
        capacity: 0,
        threshold: 0.0,
        approved: false,
        status: PeeringForecastStatus::Requested,
    }
}
