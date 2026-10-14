use super::*;

use acme_kit::text;

/// Where a lenient timeline forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientTimelineForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LenientTimelineForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LenientTimelineForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient timeline forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelineForecast {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub visible: bool,
    pub window_days: i64,
    pub unit: String,
    pub locale: String,
    pub port: i64,
    pub status: LenientTimelineForecastStatus,
}

impl LenientTimelineForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientTimelineForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientTimelineForecastStatus) -> bool {
        match self.status {
            LenientTimelineForecastStatus::Proposed => {
                return next == LenientTimelineForecastStatus::Withdrawn;
            }
            LenientTimelineForecastStatus::Approved => {
                return next == LenientTimelineForecastStatus::Rejected;
            }
            LenientTimelineForecastStatus::Rejected => {
                return next == LenientTimelineForecastStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientTimelineForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient timeline forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientTimelineForecast {
        LenientTimelineForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            visible: self.visible,
            window_days: self.window_days,
            unit: self.unit.clone(),
            locale: self.locale.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a lenient timeline forecast in status proposed.
pub fn new_lenient_timeline_forecast(code: &str, title: &str) -> LenientTimelineForecast {
    LenientTimelineForecast {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        visible: false,
        window_days: 0,
        unit: "".to_string(),
        locale: "".to_string(),
        port: 0,
        status: LenientTimelineForecastStatus::Proposed,
    }
}
