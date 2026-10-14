use super::*;

use acme_kit::text;

/// Where a redaction assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionAssignmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RedactionAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RedactionAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a redaction assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAssignment {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub attempts: i64,
    pub rotation_days: i64,
    pub issuer: String,
    pub archived: bool,
    pub status: RedactionAssignmentStatus,
}

impl RedactionAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionAssignmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionAssignmentStatus) -> bool {
        match self.status {
            RedactionAssignmentStatus::Proposed => {
                return next == RedactionAssignmentStatus::Approved;
            }
            RedactionAssignmentStatus::Approved => {
                return next == RedactionAssignmentStatus::Rejected || next == RedactionAssignmentStatus::Withdrawn;
            }
            RedactionAssignmentStatus::Rejected => {
                match next {
                    RedactionAssignmentStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RedactionAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.rotation_days as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionAssignment {
        RedactionAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            attempts: self.attempts,
            rotation_days: self.rotation_days,
            issuer: self.issuer.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a redaction assignment in status proposed.
pub fn new_redaction_assignment(code: &str, title: &str) -> RedactionAssignment {
    RedactionAssignment {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        attempts: 0,
        rotation_days: 0,
        issuer: "".to_string(),
        archived: false,
        status: RedactionAssignmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_redaction_assignment("code-166", "title-50");
        assert_eq!(item.status, RedactionAssignmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_redaction_assignment("code-166", "title-50");
        if item.move_to(RedactionAssignmentStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionAssignmentStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_redaction_assignment("code-166", "title-50");
        item.status = RedactionAssignmentStatus::Withdrawn;
        assert!(
            item.move_to(RedactionAssignmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
