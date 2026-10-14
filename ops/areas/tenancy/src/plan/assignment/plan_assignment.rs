use super::*;

use acme_kit::text;

/// Where a plan assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanAssignmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PlanAssignmentStatus {
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

impl std::fmt::Display for PlanAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a plan assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanAssignment {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub quota_units: i64,
    pub minimum: f64,
    pub status: PlanAssignmentStatus,
}

impl PlanAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanAssignmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanAssignmentStatus) -> bool {
        match self.status {
            PlanAssignmentStatus::Planned => {
                match next {
                    PlanAssignmentStatus::InProgress | PlanAssignmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanAssignmentStatus::Scheduled => {
                return next == PlanAssignmentStatus::InProgress || next == PlanAssignmentStatus::Cancelled;
            }
            PlanAssignmentStatus::InProgress => {
                match next {
                    PlanAssignmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanAssignmentStatus::Completed => {
                return next == PlanAssignmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PlanAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.quota_units as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanAssignment {
        PlanAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            quota_units: self.quota_units,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a plan assignment in status planned.
pub fn new_plan_assignment(code: &str, title: &str) -> PlanAssignment {
    PlanAssignment {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        quota_units: 0,
        minimum: 0.0,
        status: PlanAssignmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_plan_assignment("code-328", "title-923");
        assert_eq!(item.status, PlanAssignmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_plan_assignment("code-328", "title-923");
        if item.move_to(PlanAssignmentStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanAssignmentStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_plan_assignment("code-328", "title-923");
        item.status = PlanAssignmentStatus::Cancelled;
        assert!(
            item.move_to(PlanAssignmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
