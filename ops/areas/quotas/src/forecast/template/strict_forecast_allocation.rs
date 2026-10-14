use super::*;

use acme_kit::text;

/// Where a strict forecast allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictForecastAllocationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl StrictForecastAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for StrictForecastAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict forecast allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastAllocation {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub reviewer: String,
    pub grace_days: i64,
    pub margin: f64,
    pub revision: i64,
    pub sequence: i64,
    pub starts_at: i64,
    pub status: StrictForecastAllocationStatus,
}

impl StrictForecastAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictForecastAllocationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictForecastAllocationStatus) -> bool {
        match self.status {
            StrictForecastAllocationStatus::Proposed => {
                return next == StrictForecastAllocationStatus::Approved;
            }
            StrictForecastAllocationStatus::Approved => {
                return next == StrictForecastAllocationStatus::Rejected;
            }
            StrictForecastAllocationStatus::Rejected => {
                return next == StrictForecastAllocationStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictForecastAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict forecast allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.starts_at as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictForecastAllocation {
        StrictForecastAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            reviewer: self.reviewer.clone(),
            grace_days: self.grace_days,
            margin: self.margin,
            revision: self.revision,
            sequence: self.sequence,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict forecast allocation in status proposed.
pub fn new_strict_forecast_allocation(code: &str, title: &str) -> StrictForecastAllocation {
    StrictForecastAllocation {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        reviewer: "".to_string(),
        grace_days: 0,
        margin: 0.0,
        revision: 0,
        sequence: 0,
        starts_at: 0,
        status: StrictForecastAllocationStatus::Proposed,
    }
}
