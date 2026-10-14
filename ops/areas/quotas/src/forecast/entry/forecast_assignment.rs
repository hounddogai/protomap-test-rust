use super::*;

use acme_kit::text;

/// Where a forecast assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastAssignmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ForecastAssignmentStatus {
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

impl std::fmt::Display for ForecastAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One forecast assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAssignment {
    pub code: String,
    pub label: String,
    pub source: String,
    pub rotation_days: i64,
    pub quantity: i64,
    pub ttl_seconds: i64,
    pub scope: String,
    pub status: ForecastAssignmentStatus,
}

impl ForecastAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastAssignmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastAssignmentStatus) -> bool {
        match self.status {
            ForecastAssignmentStatus::Planned => {
                return next == ForecastAssignmentStatus::Scheduled || next == ForecastAssignmentStatus::Cancelled;
            }
            ForecastAssignmentStatus::Scheduled => {
                return next == ForecastAssignmentStatus::InProgress || next == ForecastAssignmentStatus::Completed;
            }
            ForecastAssignmentStatus::InProgress => {
                return next == ForecastAssignmentStatus::Completed || next == ForecastAssignmentStatus::Cancelled;
            }
            ForecastAssignmentStatus::Completed => {
                return next == ForecastAssignmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.quantity as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastAssignment {
        ForecastAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            rotation_days: self.rotation_days,
            quantity: self.quantity,
            ttl_seconds: self.ttl_seconds,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a forecast assignment in status planned.
pub fn new_forecast_assignment(code: &str, label: &str) -> ForecastAssignment {
    ForecastAssignment {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        rotation_days: 0,
        quantity: 0,
        ttl_seconds: 0,
        scope: "".to_string(),
        status: ForecastAssignmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_forecast_assignment("code-470", "label-867");
        assert_eq!(item.status, ForecastAssignmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_forecast_assignment("code-470", "label-867");
        if item.move_to(ForecastAssignmentStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastAssignmentStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_forecast_assignment("code-470", "label-867");
        item.status = ForecastAssignmentStatus::Cancelled;
        assert!(
            item.move_to(ForecastAssignmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
