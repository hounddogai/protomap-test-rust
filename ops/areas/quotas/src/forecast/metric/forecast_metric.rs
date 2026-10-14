use super::*;

use acme_kit::text;

/// Where a forecast metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ForecastMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for ForecastMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a forecast metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMetric {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub ttl_seconds: i64,
    pub maximum: f64,
    pub batch_size: i64,
    pub external_ref: String,
    pub archived: bool,
    pub ceiling: f64,
    pub status: ForecastMetricStatus,
}

impl ForecastMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastMetricStatus) -> bool {
        match self.status {
            ForecastMetricStatus::Open => {
                return next == ForecastMetricStatus::InReview || next == ForecastMetricStatus::Archived;
            }
            ForecastMetricStatus::InReview => {
                match next {
                    ForecastMetricStatus::Resolved | ForecastMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastMetricStatus::Resolved => {
                match next {
                    ForecastMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastMetricStatus::Reopened => {
                return next == ForecastMetricStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastMetric {
        ForecastMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            ttl_seconds: self.ttl_seconds,
            maximum: self.maximum,
            batch_size: self.batch_size,
            external_ref: self.external_ref.clone(),
            archived: self.archived,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a forecast metric in status open.
pub fn new_forecast_metric(code: &str, name: &str) -> ForecastMetric {
    ForecastMetric {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        ttl_seconds: 0,
        maximum: 0.0,
        batch_size: 0,
        external_ref: "".to_string(),
        archived: false,
        ceiling: 0.0,
        status: ForecastMetricStatus::Open,
    }
}
