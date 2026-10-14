use super::*;

use acme_kit::text;

/// Where a forecast review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastReviewStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ForecastReviewStatus {
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

impl std::fmt::Display for ForecastReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast review as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastReview {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub scope: String,
    pub cluster: String,
    pub quantity: i64,
    pub status: ForecastReviewStatus,
}

impl ForecastReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastReviewStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastReviewStatus) -> bool {
        match self.status {
            ForecastReviewStatus::Planned => {
                match next {
                    ForecastReviewStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastReviewStatus::Scheduled => {
                return next == ForecastReviewStatus::Cancelled;
            }
            ForecastReviewStatus::InProgress => {
                return next == ForecastReviewStatus::Completed || next == ForecastReviewStatus::Cancelled;
            }
            ForecastReviewStatus::Completed => {
                return next == ForecastReviewStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> ForecastReview {
        ForecastReview {
            code: self.code.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            scope: self.scope.clone(),
            cluster: self.cluster.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a forecast review in status planned.
pub fn new_forecast_review(code: &str, label: &str) -> ForecastReview {
    ForecastReview {
        code: code.to_string(),
        label: label.to_string(),
        notes: "".to_string(),
        scope: "".to_string(),
        cluster: "".to_string(),
        quantity: 0,
        status: ForecastReviewStatus::Planned,
    }
}
