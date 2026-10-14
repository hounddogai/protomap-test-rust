use super::*;

use acme_kit::text;

/// Where a slo assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloAssignmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SloAssignmentStatus {
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

impl std::fmt::Display for SloAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SloAssignment {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub segment: String,
    pub starts_at: i64,
    pub reviewer: String,
    pub status: SloAssignmentStatus,
}

impl SloAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloAssignmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloAssignmentStatus) -> bool {
        match self.status {
            SloAssignmentStatus::Planned => {
                match next {
                    SloAssignmentStatus::InProgress | SloAssignmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloAssignmentStatus::Scheduled => {
                return next == SloAssignmentStatus::InProgress || next == SloAssignmentStatus::Cancelled;
            }
            SloAssignmentStatus::InProgress => {
                return next == SloAssignmentStatus::Completed;
            }
            SloAssignmentStatus::Completed => {
                match next {
                    SloAssignmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: SloAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "slo assignment", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.amount * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloAssignment {
        SloAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            segment: self.segment.clone(),
            starts_at: self.starts_at,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a slo assignment in status planned.
pub fn new_slo_assignment(code: &str, title: &str) -> SloAssignment {
    SloAssignment {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        segment: "".to_string(),
        starts_at: 0,
        reviewer: "".to_string(),
        status: SloAssignmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_slo_assignment("code-231", "title-369");
        assert_eq!(item.status, SloAssignmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_slo_assignment("code-231", "title-369");
        if item.move_to(SloAssignmentStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloAssignmentStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_slo_assignment("code-231", "title-369");
        item.status = SloAssignmentStatus::Cancelled;
        assert!(
            item.move_to(SloAssignmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
