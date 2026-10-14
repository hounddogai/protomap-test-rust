use super::*;

use acme_kit::text;

/// Where a forecast plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastPlanStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ForecastPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ForecastPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast plan as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastPlan {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub batch_size: i64,
    pub threshold: f64,
    pub pinned: bool,
    pub discount: f64,
    pub minimum: f64,
    pub capacity: i64,
    pub status: ForecastPlanStatus,
}

impl ForecastPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastPlanStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastPlanStatus) -> bool {
        match self.status {
            ForecastPlanStatus::Created => {
                return next == ForecastPlanStatus::Published;
            }
            ForecastPlanStatus::Validated => {
                match next {
                    ForecastPlanStatus::Published | ForecastPlanStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastPlanStatus::Published => {
                return next == ForecastPlanStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.minimum * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastPlan {
        ForecastPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            batch_size: self.batch_size,
            threshold: self.threshold,
            pinned: self.pinned,
            discount: self.discount,
            minimum: self.minimum,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a forecast plan in status created.
pub fn new_forecast_plan(code: &str, label: &str) -> ForecastPlan {
    ForecastPlan {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        batch_size: 0,
        threshold: 0.0,
        pinned: false,
        discount: 0.0,
        minimum: 0.0,
        capacity: 0,
        status: ForecastPlanStatus::Created,
    }
}
