use super::*;

use acme_kit::text;

/// Where a pending slo forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingSloForecastStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PendingSloForecastStatus {
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

impl std::fmt::Display for PendingSloForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending slo forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloForecast {
    pub code: String,
    pub title: String,
    pub category: String,
    pub checksum: String,
    pub quantity: i64,
    pub status: PendingSloForecastStatus,
}

impl PendingSloForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingSloForecastStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingSloForecastStatus) -> bool {
        match self.status {
            PendingSloForecastStatus::Planned => {
                return next == PendingSloForecastStatus::Cancelled;
            }
            PendingSloForecastStatus::Scheduled => {
                match next {
                    PendingSloForecastStatus::Completed | PendingSloForecastStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSloForecastStatus::InProgress => {
                return next == PendingSloForecastStatus::Cancelled;
            }
            PendingSloForecastStatus::Completed => {
                match next {
                    PendingSloForecastStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PendingSloForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending slo forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> PendingSloForecast {
        PendingSloForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            checksum: self.checksum.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a pending slo forecast in status planned.
pub fn new_pending_slo_forecast(code: &str, title: &str) -> PendingSloForecast {
    PendingSloForecast {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        checksum: "".to_string(),
        quantity: 0,
        status: PendingSloForecastStatus::Planned,
    }
}
