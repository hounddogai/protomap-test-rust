use super::*;

use acme_kit::text;

/// Where a lenient binding assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientBindingAssignmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientBindingAssignmentStatus {
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

impl std::fmt::Display for LenientBindingAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient binding assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingAssignment {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub title: String,
    pub capacity: i64,
    pub version: i64,
    pub priority: i64,
    pub duration_minutes: i64,
    pub status: LenientBindingAssignmentStatus,
}

impl LenientBindingAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientBindingAssignmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientBindingAssignmentStatus) -> bool {
        match self.status {
            LenientBindingAssignmentStatus::Planned => {
                return next == LenientBindingAssignmentStatus::Scheduled
                    || next == LenientBindingAssignmentStatus::Completed;
            }
            LenientBindingAssignmentStatus::Scheduled => {
                return next == LenientBindingAssignmentStatus::InProgress
                    || next == LenientBindingAssignmentStatus::Cancelled;
            }
            LenientBindingAssignmentStatus::InProgress => {
                match next {
                    LenientBindingAssignmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientBindingAssignmentStatus::Completed => {
                return next == LenientBindingAssignmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientBindingAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient binding assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient binding assignment", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.capacity as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientBindingAssignment {
        LenientBindingAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            title: self.title.clone(),
            capacity: self.capacity,
            version: self.version,
            priority: self.priority,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient binding assignment in status planned.
pub fn new_lenient_binding_assignment(code: &str, label: &str) -> LenientBindingAssignment {
    LenientBindingAssignment {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        title: "".to_string(),
        capacity: 0,
        version: 0,
        priority: 0,
        duration_minutes: 0,
        status: LenientBindingAssignmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_binding_assignment("code-850", "label-878");
        assert_eq!(item.status, LenientBindingAssignmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_lenient_binding_assignment("code-850", "label-878");
        if item.move_to(LenientBindingAssignmentStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientBindingAssignmentStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_binding_assignment("code-850", "label-878");
        item.status = LenientBindingAssignmentStatus::Cancelled;
        assert!(
            item.move_to(LenientBindingAssignmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
