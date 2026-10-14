use super::*;

use acme_kit::text;

/// Where a quote forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteForecastStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl QuoteForecastStatus {
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

impl std::fmt::Display for QuoteForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteForecast {
    pub code: String,
    pub label: String,
    pub source: String,
    pub score: f64,
    pub reviewer: String,
    pub phone: String,
    pub notes: String,
    pub limit: i64,
    pub threshold: f64,
    pub status: QuoteForecastStatus,
}

impl QuoteForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteForecastStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteForecastStatus) -> bool {
        match self.status {
            QuoteForecastStatus::Planned => {
                return next == QuoteForecastStatus::Completed;
            }
            QuoteForecastStatus::Scheduled => {
                return next == QuoteForecastStatus::Completed || next == QuoteForecastStatus::Cancelled;
            }
            QuoteForecastStatus::InProgress => {
                return next == QuoteForecastStatus::Completed || next == QuoteForecastStatus::Cancelled;
            }
            QuoteForecastStatus::Completed => {
                return next == QuoteForecastStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuoteForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.score + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteForecast {
        QuoteForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            score: self.score,
            reviewer: self.reviewer.clone(),
            phone: self.phone.clone(),
            notes: self.notes.clone(),
            limit: self.limit,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a quote forecast in status planned.
pub fn new_quote_forecast(code: &str, label: &str) -> QuoteForecast {
    QuoteForecast {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        score: 0.0,
        reviewer: "".to_string(),
        phone: "".to_string(),
        notes: "".to_string(),
        limit: 0,
        threshold: 0.0,
        status: QuoteForecastStatus::Planned,
    }
}
