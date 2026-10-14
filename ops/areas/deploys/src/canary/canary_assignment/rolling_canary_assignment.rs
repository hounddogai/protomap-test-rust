use super::*;

use acme_kit::text;

/// Where a rolling canary assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingCanaryAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingCanaryAssignmentStatus {
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

impl std::fmt::Display for RollingCanaryAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling canary assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryAssignment {
    pub code: String,
    pub label: String,
    pub category: String,
    pub visible: bool,
    pub scope: String,
    pub amount: f64,
    pub status: RollingCanaryAssignmentStatus,
}

impl RollingCanaryAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingCanaryAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingCanaryAssignmentStatus) -> bool {
        match self.status {
            RollingCanaryAssignmentStatus::Open => {
                return next == RollingCanaryAssignmentStatus::InReview
                    || next == RollingCanaryAssignmentStatus::Resolved;
            }
            RollingCanaryAssignmentStatus::InReview => {
                return next == RollingCanaryAssignmentStatus::Resolved;
            }
            RollingCanaryAssignmentStatus::Resolved => {
                return next == RollingCanaryAssignmentStatus::Archived;
            }
            RollingCanaryAssignmentStatus::Reopened => {
                return next == RollingCanaryAssignmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingCanaryAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling canary assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> RollingCanaryAssignment {
        RollingCanaryAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            visible: self.visible,
            scope: self.scope.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a rolling canary assignment in status open.
pub fn new_rolling_canary_assignment(code: &str, label: &str) -> RollingCanaryAssignment {
    RollingCanaryAssignment {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        visible: false,
        scope: "".to_string(),
        amount: 0.0,
        status: RollingCanaryAssignmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rolling_canary_assignment("code-524", "label-306");
        assert_eq!(item.status, RollingCanaryAssignmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_rolling_canary_assignment("code-524", "label-306");
        if item.move_to(RollingCanaryAssignmentStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingCanaryAssignmentStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rolling_canary_assignment("code-524", "label-306");
        item.status = RollingCanaryAssignmentStatus::Archived;
        assert!(
            item.move_to(RollingCanaryAssignmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
