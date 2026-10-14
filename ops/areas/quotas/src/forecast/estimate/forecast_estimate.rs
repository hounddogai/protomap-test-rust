use super::*;

use acme_kit::text;

/// Where a forecast estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ForecastEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ForecastEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastEstimate {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub created_at: i64,
    pub duration_minutes: i64,
    pub capacity: i64,
    pub grace_days: i64,
    pub region: String,
    pub threshold: f64,
    pub status: ForecastEstimateStatus,
}

impl ForecastEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastEstimateStatus) -> bool {
        match self.status {
            ForecastEstimateStatus::Created => {
                match next {
                    ForecastEstimateStatus::Validated | ForecastEstimateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastEstimateStatus::Validated => {
                return next == ForecastEstimateStatus::Published;
            }
            ForecastEstimateStatus::Published => {
                return next == ForecastEstimateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.threshold + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastEstimate {
        ForecastEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            created_at: self.created_at,
            duration_minutes: self.duration_minutes,
            capacity: self.capacity,
            grace_days: self.grace_days,
            region: self.region.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a forecast estimate in status created.
pub fn new_forecast_estimate(code: &str, title: &str) -> ForecastEstimate {
    ForecastEstimate {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        created_at: 0,
        duration_minutes: 0,
        capacity: 0,
        grace_days: 0,
        region: "".to_string(),
        threshold: 0.0,
        status: ForecastEstimateStatus::Created,
    }
}
