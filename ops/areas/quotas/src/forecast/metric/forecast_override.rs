use super::*;

use acme_kit::text;

/// Where a forecast override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastOverrideStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ForecastOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ForecastOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One forecast override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastOverride {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub rate: f64,
    pub updated_at: i64,
    pub window_days: i64,
    pub status: ForecastOverrideStatus,
}

impl ForecastOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastOverrideStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastOverrideStatus) -> bool {
        match self.status {
            ForecastOverrideStatus::Created => {
                return next == ForecastOverrideStatus::Validated || next == ForecastOverrideStatus::Retired;
            }
            ForecastOverrideStatus::Validated => {
                match next {
                    ForecastOverrideStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastOverrideStatus::Published => {
                match next {
                    ForecastOverrideStatus::Retired => {
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

    pub fn move_to(&mut self, next: ForecastOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast override {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.updated_at as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastOverride {
        ForecastOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            rate: self.rate,
            updated_at: self.updated_at,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a forecast override in status created.
pub fn new_forecast_override(code: &str, title: &str) -> ForecastOverride {
    ForecastOverride {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        rate: 0.0,
        updated_at: 0,
        window_days: 0,
        status: ForecastOverrideStatus::Created,
    }
}
