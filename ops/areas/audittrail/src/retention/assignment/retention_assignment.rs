use super::*;

use acme_kit::text;

/// Where a retention assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RetentionAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RetentionAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAssignment {
    pub code: String,
    pub name: String,
    pub category: String,
    pub ends_at: i64,
    pub namespace: String,
    pub status: RetentionAssignmentStatus,
}

impl RetentionAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionAssignmentStatus) -> bool {
        match self.status {
            RetentionAssignmentStatus::Open => {
                match next {
                    RetentionAssignmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionAssignmentStatus::InReview => {
                return next == RetentionAssignmentStatus::Resolved;
            }
            RetentionAssignmentStatus::Resolved => {
                return next == RetentionAssignmentStatus::Archived;
            }
            RetentionAssignmentStatus::Reopened => {
                return next == RetentionAssignmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> RetentionAssignment {
        RetentionAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            ends_at: self.ends_at,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a retention assignment in status open.
pub fn new_retention_assignment(code: &str, name: &str) -> RetentionAssignment {
    RetentionAssignment {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        ends_at: 0,
        namespace: "".to_string(),
        status: RetentionAssignmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_retention_assignment("code-386", "name-998");
        assert_eq!(item.status, RetentionAssignmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_retention_assignment("code-386", "name-998");
        if item.move_to(RetentionAssignmentStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionAssignmentStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_retention_assignment("code-386", "name-998");
        item.status = RetentionAssignmentStatus::Archived;
        assert!(
            item.move_to(RetentionAssignmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
