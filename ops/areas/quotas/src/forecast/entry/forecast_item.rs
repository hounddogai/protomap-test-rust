use super::*;

use acme_kit::text;

/// Where a forecast item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastItemStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ForecastItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for ForecastItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One forecast item, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastItem {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub host: String,
    pub audience: String,
    pub duration_minutes: i64,
    pub limit: i64,
    pub archived: bool,
    pub status: ForecastItemStatus,
}

impl ForecastItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastItemStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastItemStatus) -> bool {
        match self.status {
            ForecastItemStatus::Planned => {
                match next {
                    ForecastItemStatus::Scheduled | ForecastItemStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastItemStatus::Scheduled => {
                match next {
                    ForecastItemStatus::Completed | ForecastItemStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastItemStatus::InProgress => {
                return next == ForecastItemStatus::Cancelled;
            }
            ForecastItemStatus::Completed => {
                match next {
                    ForecastItemStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ForecastItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.limit as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastItem {
        ForecastItem {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            host: self.host.clone(),
            audience: self.audience.clone(),
            duration_minutes: self.duration_minutes,
            limit: self.limit,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a forecast item in status planned.
pub fn new_forecast_item(code: &str, label: &str) -> ForecastItem {
    ForecastItem {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        host: "".to_string(),
        audience: "".to_string(),
        duration_minutes: 0,
        limit: 0,
        archived: false,
        status: ForecastItemStatus::Planned,
    }
}
