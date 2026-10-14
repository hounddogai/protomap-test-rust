use super::*;

use acme_kit::text;

/// Where a runbook assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookAssignmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RunbookAssignmentStatus {
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

impl std::fmt::Display for RunbookAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One runbook assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAssignment {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub window_days: i64,
    pub phone: String,
    pub status: RunbookAssignmentStatus,
}

impl RunbookAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookAssignmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookAssignmentStatus) -> bool {
        match self.status {
            RunbookAssignmentStatus::Planned => {
                match next {
                    RunbookAssignmentStatus::Scheduled | RunbookAssignmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookAssignmentStatus::Scheduled => {
                match next {
                    RunbookAssignmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookAssignmentStatus::InProgress => {
                match next {
                    RunbookAssignmentStatus::Completed | RunbookAssignmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookAssignmentStatus::Completed => {
                match next {
                    RunbookAssignmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RunbookAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "runbook assignment", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        (self.weight * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookAssignment {
        RunbookAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            window_days: self.window_days,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a runbook assignment in status planned.
pub fn new_runbook_assignment(code: &str, title: &str) -> RunbookAssignment {
    RunbookAssignment {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        window_days: 0,
        phone: "".to_string(),
        status: RunbookAssignmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_runbook_assignment("code-937", "title-46");
        assert_eq!(item.status, RunbookAssignmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_runbook_assignment("code-937", "title-46");
        if item.move_to(RunbookAssignmentStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookAssignmentStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_runbook_assignment("code-937", "title-46");
        item.status = RunbookAssignmentStatus::Cancelled;
        assert!(
            item.move_to(RunbookAssignmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
