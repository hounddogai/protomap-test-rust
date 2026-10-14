use super::*;

use acme_kit::text;

/// Where a forecast segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastSegmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ForecastSegmentStatus {
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

impl std::fmt::Display for ForecastSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One forecast segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSegment {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub phone: String,
    pub budget: f64,
    pub notes: String,
    pub status: ForecastSegmentStatus,
}

impl ForecastSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastSegmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastSegmentStatus) -> bool {
        match self.status {
            ForecastSegmentStatus::Draft => {
                match next {
                    ForecastSegmentStatus::Pending | ForecastSegmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastSegmentStatus::Pending => {
                match next {
                    ForecastSegmentStatus::Active | ForecastSegmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastSegmentStatus::Active => {
                return next == ForecastSegmentStatus::Closed;
            }
            ForecastSegmentStatus::Suspended => {
                match next {
                    ForecastSegmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: ForecastSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.quota_units as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastSegment {
        ForecastSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            phone: self.phone.clone(),
            budget: self.budget,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a forecast segment in status draft.
pub fn new_forecast_segment(code: &str, title: &str) -> ForecastSegment {
    ForecastSegment {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        phone: "".to_string(),
        budget: 0.0,
        notes: "".to_string(),
        status: ForecastSegmentStatus::Draft,
    }
}
