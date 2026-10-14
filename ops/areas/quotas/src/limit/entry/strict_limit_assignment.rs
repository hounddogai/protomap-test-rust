use super::*;

use acme_kit::text;

/// Where a strict limit assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictLimitAssignmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl StrictLimitAssignmentStatus {
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

impl std::fmt::Display for StrictLimitAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict limit assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictLimitAssignment {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub tenant_id: String,
    pub archived: bool,
    pub sequence: i64,
    pub cipher: String,
    pub status: StrictLimitAssignmentStatus,
}

impl StrictLimitAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictLimitAssignmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictLimitAssignmentStatus) -> bool {
        match self.status {
            StrictLimitAssignmentStatus::Planned => {
                return next == StrictLimitAssignmentStatus::Completed
                    || next == StrictLimitAssignmentStatus::Cancelled;
            }
            StrictLimitAssignmentStatus::Scheduled => {
                return next == StrictLimitAssignmentStatus::Completed
                    || next == StrictLimitAssignmentStatus::Cancelled;
            }
            StrictLimitAssignmentStatus::InProgress => {
                return next == StrictLimitAssignmentStatus::Completed
                    || next == StrictLimitAssignmentStatus::Cancelled;
            }
            StrictLimitAssignmentStatus::Completed => {
                return next == StrictLimitAssignmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictLimitAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict limit assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict limit assignment", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.ceiling - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictLimitAssignment {
        StrictLimitAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            tenant_id: self.tenant_id.clone(),
            archived: self.archived,
            sequence: self.sequence,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict limit assignment in status planned.
pub fn new_strict_limit_assignment(code: &str, title: &str) -> StrictLimitAssignment {
    StrictLimitAssignment {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        tenant_id: "".to_string(),
        archived: false,
        sequence: 0,
        cipher: "".to_string(),
        status: StrictLimitAssignmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_strict_limit_assignment("code-490", "title-554");
        assert_eq!(item.status, StrictLimitAssignmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_strict_limit_assignment("code-490", "title-554");
        if item.move_to(StrictLimitAssignmentStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictLimitAssignmentStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_strict_limit_assignment("code-490", "title-554");
        item.status = StrictLimitAssignmentStatus::Cancelled;
        assert!(
            item.move_to(StrictLimitAssignmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
