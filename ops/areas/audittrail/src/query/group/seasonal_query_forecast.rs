use super::*;

use acme_kit::text;

/// Where a seasonal query forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalQueryForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalQueryForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalQueryForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal query forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryForecast {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub priority: i64,
    pub burst: i64,
    pub key_id: String,
    pub status: SeasonalQueryForecastStatus,
}

impl SeasonalQueryForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalQueryForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalQueryForecastStatus) -> bool {
        match self.status {
            SeasonalQueryForecastStatus::Proposed => {
                return next == SeasonalQueryForecastStatus::Approved;
            }
            SeasonalQueryForecastStatus::Approved => {
                return next == SeasonalQueryForecastStatus::Rejected;
            }
            SeasonalQueryForecastStatus::Rejected => {
                return next == SeasonalQueryForecastStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalQueryForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal query forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.burst as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalQueryForecast {
        SeasonalQueryForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            priority: self.priority,
            burst: self.burst,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal query forecast in status proposed.
pub fn new_seasonal_query_forecast(code: &str, name: &str) -> SeasonalQueryForecast {
    SeasonalQueryForecast {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        priority: 0,
        burst: 0,
        key_id: "".to_string(),
        status: SeasonalQueryForecastStatus::Proposed,
    }
}
