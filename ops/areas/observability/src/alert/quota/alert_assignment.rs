use super::*;

use acme_kit::text;

/// Where a alert assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AlertAssignmentStatus {
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

impl std::fmt::Display for AlertAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a alert assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertAssignment {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub maximum: f64,
    pub threshold: f64,
    pub limit: i64,
    pub ttl_seconds: i64,
    pub channel: String,
    pub notes: String,
    pub status: AlertAssignmentStatus,
}

impl AlertAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertAssignmentStatus) -> bool {
        match self.status {
            AlertAssignmentStatus::Open => {
                return next == AlertAssignmentStatus::Resolved;
            }
            AlertAssignmentStatus::InReview => {
                match next {
                    AlertAssignmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertAssignmentStatus::Resolved => {
                return next == AlertAssignmentStatus::Archived;
            }
            AlertAssignmentStatus::Reopened => {
                match next {
                    AlertAssignmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: AlertAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.maximum * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertAssignment {
        AlertAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            maximum: self.maximum,
            threshold: self.threshold,
            limit: self.limit,
            ttl_seconds: self.ttl_seconds,
            channel: self.channel.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a alert assignment in status open.
pub fn new_alert_assignment(code: &str, title: &str) -> AlertAssignment {
    AlertAssignment {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        maximum: 0.0,
        threshold: 0.0,
        limit: 0,
        ttl_seconds: 0,
        channel: "".to_string(),
        notes: "".to_string(),
        status: AlertAssignmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_alert_assignment("code-198", "title-357");
        assert_eq!(item.status, AlertAssignmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_alert_assignment("code-198", "title-357");
        if item.move_to(AlertAssignmentStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertAssignmentStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_alert_assignment("code-198", "title-357");
        item.status = AlertAssignmentStatus::Archived;
        assert!(
            item.move_to(AlertAssignmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
