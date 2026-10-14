use super::*;

use acme_kit::text;

/// Where a forecast entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastEntryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ForecastEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ForecastEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a forecast entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastEntry {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub unit: String,
    pub quantity: i64,
    pub approved: bool,
    pub priority: i64,
    pub status: ForecastEntryStatus,
}

impl ForecastEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastEntryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastEntryStatus) -> bool {
        match self.status {
            ForecastEntryStatus::Created => {
                return next == ForecastEntryStatus::Validated;
            }
            ForecastEntryStatus::Validated => {
                match next {
                    ForecastEntryStatus::Published | ForecastEntryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastEntryStatus::Published => {
                match next {
                    ForecastEntryStatus::Retired => {
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

    pub fn move_to(&mut self, next: ForecastEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.priority as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastEntry {
        ForecastEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            unit: self.unit.clone(),
            quantity: self.quantity,
            approved: self.approved,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a forecast entry in status created.
pub fn new_forecast_entry(code: &str, label: &str) -> ForecastEntry {
    ForecastEntry {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        unit: "".to_string(),
        quantity: 0,
        approved: false,
        priority: 0,
        status: ForecastEntryStatus::Created,
    }
}
