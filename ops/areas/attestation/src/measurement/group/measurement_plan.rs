use super::*;

use acme_kit::text;

/// Where a measurement plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementPlanStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl MeasurementPlanStatus {
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

impl std::fmt::Display for MeasurementPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement plan as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPlan {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub threshold: f64,
    pub source: String,
    pub status: MeasurementPlanStatus,
}

impl MeasurementPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementPlanStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementPlanStatus) -> bool {
        match self.status {
            MeasurementPlanStatus::Planned => {
                return next == MeasurementPlanStatus::Cancelled;
            }
            MeasurementPlanStatus::Scheduled => {
                match next {
                    MeasurementPlanStatus::Completed | MeasurementPlanStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementPlanStatus::InProgress => {
                return next == MeasurementPlanStatus::Completed || next == MeasurementPlanStatus::Cancelled;
            }
            MeasurementPlanStatus::Completed => {
                return next == MeasurementPlanStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "measurement plan", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.priority as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementPlan {
        MeasurementPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            threshold: self.threshold,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a measurement plan in status planned.
pub fn new_measurement_plan(code: &str, title: &str) -> MeasurementPlan {
    MeasurementPlan {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        threshold: 0.0,
        source: "".to_string(),
        status: MeasurementPlanStatus::Planned,
    }
}
