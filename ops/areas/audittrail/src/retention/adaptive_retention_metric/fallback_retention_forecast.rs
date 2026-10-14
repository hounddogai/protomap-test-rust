use super::*;

use acme_kit::text;

/// Where a fallback retention forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRetentionForecastStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackRetentionForecastStatus {
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

impl std::fmt::Display for FallbackRetentionForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback retention forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRetentionForecast {
    pub code: String,
    pub label: String,
    pub host: String,
    pub owner_id: String,
    pub cipher: String,
    pub scope: String,
    pub reviewer: String,
    pub quantity: i64,
    pub status: FallbackRetentionForecastStatus,
}

impl FallbackRetentionForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRetentionForecastStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRetentionForecastStatus) -> bool {
        match self.status {
            FallbackRetentionForecastStatus::Planned => {
                match next {
                    FallbackRetentionForecastStatus::Scheduled | FallbackRetentionForecastStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRetentionForecastStatus::Scheduled => {
                match next {
                    FallbackRetentionForecastStatus::InProgress | FallbackRetentionForecastStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRetentionForecastStatus::InProgress => {
                return next == FallbackRetentionForecastStatus::Completed
                    || next == FallbackRetentionForecastStatus::Cancelled;
            }
            FallbackRetentionForecastStatus::Completed => {
                match next {
                    FallbackRetentionForecastStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: FallbackRetentionForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback retention forecast {} cannot move from {} to {}",
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
        self.quantity as f64
    }

    pub fn copy(&self) -> FallbackRetentionForecast {
        FallbackRetentionForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            owner_id: self.owner_id.clone(),
            cipher: self.cipher.clone(),
            scope: self.scope.clone(),
            reviewer: self.reviewer.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a fallback retention forecast in status planned.
pub fn new_fallback_retention_forecast(code: &str, label: &str) -> FallbackRetentionForecast {
    FallbackRetentionForecast {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        owner_id: "".to_string(),
        cipher: "".to_string(),
        scope: "".to_string(),
        reviewer: "".to_string(),
        quantity: 0,
        status: FallbackRetentionForecastStatus::Planned,
    }
}
