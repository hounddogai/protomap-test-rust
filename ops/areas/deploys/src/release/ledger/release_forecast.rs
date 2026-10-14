use super::*;

use acme_kit::text;

/// Where a release forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseForecastStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ReleaseForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ReleaseForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a release forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseForecast {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub weight: f64,
    pub margin: f64,
    pub unit: String,
    pub cipher: String,
    pub principal: String,
    pub external_ref: String,
    pub quota_units: i64,
    pub reviewer: String,
    pub status: ReleaseForecastStatus,
}

impl ReleaseForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseForecastStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseForecastStatus) -> bool {
        match self.status {
            ReleaseForecastStatus::Draft => {
                match next {
                    ReleaseForecastStatus::Pending | ReleaseForecastStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseForecastStatus::Pending => {
                match next {
                    ReleaseForecastStatus::Active | ReleaseForecastStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseForecastStatus::Active => {
                match next {
                    ReleaseForecastStatus::Suspended | ReleaseForecastStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseForecastStatus::Suspended => {
                return next == ReleaseForecastStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.priority as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseForecast {
        ReleaseForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            weight: self.weight,
            margin: self.margin,
            unit: self.unit.clone(),
            cipher: self.cipher.clone(),
            principal: self.principal.clone(),
            external_ref: self.external_ref.clone(),
            quota_units: self.quota_units,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a release forecast in status draft.
pub fn new_release_forecast(code: &str, title: &str) -> ReleaseForecast {
    ReleaseForecast {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        weight: 0.0,
        margin: 0.0,
        unit: "".to_string(),
        cipher: "".to_string(),
        principal: "".to_string(),
        external_ref: "".to_string(),
        quota_units: 0,
        reviewer: "".to_string(),
        status: ReleaseForecastStatus::Draft,
    }
}
