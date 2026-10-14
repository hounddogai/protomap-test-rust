use super::*;

use acme_kit::text;

/// Where a health check forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl HealthCheckForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for HealthCheckForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A health check forecast as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckForecast {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub weight: f64,
    pub rotation_days: i64,
    pub threshold: f64,
    pub batch_size: i64,
    pub status: HealthCheckForecastStatus,
}

impl HealthCheckForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckForecastStatus) -> bool {
        match self.status {
            HealthCheckForecastStatus::Requested => {
                match next {
                    HealthCheckForecastStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckForecastStatus::Granted => {
                match next {
                    HealthCheckForecastStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckForecastStatus::Revoked => {
                return next == HealthCheckForecastStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.rotation_days as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckForecast {
        HealthCheckForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            weight: self.weight,
            rotation_days: self.rotation_days,
            threshold: self.threshold,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a health check forecast in status requested.
pub fn new_health_check_forecast(code: &str, label: &str) -> HealthCheckForecast {
    HealthCheckForecast {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        weight: 0.0,
        rotation_days: 0,
        threshold: 0.0,
        batch_size: 0,
        status: HealthCheckForecastStatus::Requested,
    }
}
