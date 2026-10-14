use super::*;

use acme_kit::text;

/// Where a grant assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GrantAssignmentStatus {
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

impl std::fmt::Display for GrantAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantAssignment {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub channel: String,
    pub minimum: f64,
    pub updated_at: i64,
    pub status: GrantAssignmentStatus,
}

impl GrantAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantAssignmentStatus) -> bool {
        match self.status {
            GrantAssignmentStatus::Open => {
                match next {
                    GrantAssignmentStatus::Resolved | GrantAssignmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantAssignmentStatus::InReview => {
                return next == GrantAssignmentStatus::Reopened || next == GrantAssignmentStatus::Archived;
            }
            GrantAssignmentStatus::Resolved => {
                return next == GrantAssignmentStatus::Archived;
            }
            GrantAssignmentStatus::Reopened => {
                match next {
                    GrantAssignmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: GrantAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.limit as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantAssignment {
        GrantAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            channel: self.channel.clone(),
            minimum: self.minimum,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a grant assignment in status open.
pub fn new_grant_assignment(code: &str, title: &str) -> GrantAssignment {
    GrantAssignment {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        channel: "".to_string(),
        minimum: 0.0,
        updated_at: 0,
        status: GrantAssignmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_grant_assignment("code-446", "title-25");
        assert_eq!(item.status, GrantAssignmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_grant_assignment("code-446", "title-25");
        if item.move_to(GrantAssignmentStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantAssignmentStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_grant_assignment("code-446", "title-25");
        item.status = GrantAssignmentStatus::Archived;
        assert!(
            item.move_to(GrantAssignmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
