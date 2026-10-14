use super::*;

use acme_kit::text;

/// Where a measurement review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl MeasurementReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for MeasurementReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a measurement review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementReview {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub quota_units: i64,
    pub category: String,
    pub minimum: f64,
    pub status: MeasurementReviewStatus,
}

impl MeasurementReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementReviewStatus) -> bool {
        match self.status {
            MeasurementReviewStatus::Idle => {
                match next {
                    MeasurementReviewStatus::Ready | MeasurementReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementReviewStatus::Warming => {
                match next {
                    MeasurementReviewStatus::Ready | MeasurementReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementReviewStatus::Ready => {
                return next == MeasurementReviewStatus::Stopped;
            }
            MeasurementReviewStatus::Draining => {
                return next == MeasurementReviewStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.quota_units as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementReview {
        MeasurementReview {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            quota_units: self.quota_units,
            category: self.category.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a measurement review in status idle.
pub fn new_measurement_review(code: &str, label: &str) -> MeasurementReview {
    MeasurementReview {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        quota_units: 0,
        category: "".to_string(),
        minimum: 0.0,
        status: MeasurementReviewStatus::Idle,
    }
}
