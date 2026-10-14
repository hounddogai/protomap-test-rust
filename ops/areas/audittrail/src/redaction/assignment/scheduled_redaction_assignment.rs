use super::*;

use acme_kit::text;

/// Where a scheduled redaction assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledRedactionAssignmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledRedactionAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledRedactionAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled redaction assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRedactionAssignment {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub revision: i64,
    pub maximum: f64,
    pub window_days: i64,
    pub ceiling: f64,
    pub budget: f64,
    pub updated_at: i64,
    pub score: f64,
    pub status: ScheduledRedactionAssignmentStatus,
}

impl ScheduledRedactionAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledRedactionAssignmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledRedactionAssignmentStatus) -> bool {
        match self.status {
            ScheduledRedactionAssignmentStatus::Requested => {
                match next {
                    ScheduledRedactionAssignmentStatus::Granted | ScheduledRedactionAssignmentStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRedactionAssignmentStatus::Granted => {
                return next == ScheduledRedactionAssignmentStatus::Revoked
                    || next == ScheduledRedactionAssignmentStatus::Expired;
            }
            ScheduledRedactionAssignmentStatus::Revoked => {
                return next == ScheduledRedactionAssignmentStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledRedactionAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled redaction assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled redaction assignment", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ceiling - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledRedactionAssignment {
        ScheduledRedactionAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            revision: self.revision,
            maximum: self.maximum,
            window_days: self.window_days,
            ceiling: self.ceiling,
            budget: self.budget,
            updated_at: self.updated_at,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled redaction assignment in status requested.
pub fn new_scheduled_redaction_assignment(code: &str, name: &str) -> ScheduledRedactionAssignment {
    ScheduledRedactionAssignment {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        revision: 0,
        maximum: 0.0,
        window_days: 0,
        ceiling: 0.0,
        budget: 0.0,
        updated_at: 0,
        score: 0.0,
        status: ScheduledRedactionAssignmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_scheduled_redaction_assignment("code-639", "name-595");
        assert_eq!(item.status, ScheduledRedactionAssignmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_scheduled_redaction_assignment("code-639", "name-595");
        if item.move_to(ScheduledRedactionAssignmentStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledRedactionAssignmentStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_scheduled_redaction_assignment("code-639", "name-595");
        item.status = ScheduledRedactionAssignmentStatus::Expired;
        assert!(
            item.move_to(ScheduledRedactionAssignmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
