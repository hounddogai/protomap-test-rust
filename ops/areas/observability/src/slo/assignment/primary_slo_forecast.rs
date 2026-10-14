use super::*;

use acme_kit::text;

/// Where a primary slo forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySloForecastStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimarySloForecastStatus {
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

impl std::fmt::Display for PrimarySloForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary slo forecast as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloForecast {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub segment: String,
    pub notes: String,
    pub grace_days: i64,
    pub attempts: i64,
    pub audience: String,
    pub status: PrimarySloForecastStatus,
}

impl PrimarySloForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySloForecastStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySloForecastStatus) -> bool {
        match self.status {
            PrimarySloForecastStatus::Open => {
                return next == PrimarySloForecastStatus::InReview;
            }
            PrimarySloForecastStatus::InReview => {
                return next == PrimarySloForecastStatus::Resolved || next == PrimarySloForecastStatus::Reopened;
            }
            PrimarySloForecastStatus::Resolved => {
                return next == PrimarySloForecastStatus::Reopened || next == PrimarySloForecastStatus::Archived;
            }
            PrimarySloForecastStatus::Reopened => {
                match next {
                    PrimarySloForecastStatus::Archived => {
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

    pub fn move_to(&mut self, next: PrimarySloForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary slo forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary slo forecast", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.attempts as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimarySloForecast {
        PrimarySloForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            segment: self.segment.clone(),
            notes: self.notes.clone(),
            grace_days: self.grace_days,
            attempts: self.attempts,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a primary slo forecast in status open.
pub fn new_primary_slo_forecast(code: &str, title: &str) -> PrimarySloForecast {
    PrimarySloForecast {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        segment: "".to_string(),
        notes: "".to_string(),
        grace_days: 0,
        attempts: 0,
        audience: "".to_string(),
        status: PrimarySloForecastStatus::Open,
    }
}
