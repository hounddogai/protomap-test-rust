use super::*;

use acme_kit::text;

/// Where a retention forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionForecastStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RetentionForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RetentionForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a retention forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionForecast {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub batch_size: i64,
    pub name: String,
    pub status: RetentionForecastStatus,
}

impl RetentionForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionForecastStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionForecastStatus) -> bool {
        match self.status {
            RetentionForecastStatus::Created => {
                match next {
                    RetentionForecastStatus::Published | RetentionForecastStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionForecastStatus::Validated => {
                return next == RetentionForecastStatus::Published || next == RetentionForecastStatus::Retired;
            }
            RetentionForecastStatus::Published => {
                match next {
                    RetentionForecastStatus::Retired => {
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

    pub fn move_to(&mut self, next: RetentionForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.batch_size as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionForecast {
        RetentionForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            batch_size: self.batch_size,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a retention forecast in status created.
pub fn new_retention_forecast(code: &str, label: &str) -> RetentionForecast {
    RetentionForecast {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        batch_size: 0,
        name: "".to_string(),
        status: RetentionForecastStatus::Created,
    }
}
