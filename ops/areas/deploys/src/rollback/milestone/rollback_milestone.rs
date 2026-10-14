use super::*;

use acme_kit::text;

/// Where a rollback milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackMilestoneStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollbackMilestoneStatus {
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

impl std::fmt::Display for RollbackMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rollback milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackMilestone {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub category: String,
    pub margin: f64,
    pub cost: f64,
    pub capacity: i64,
    pub rate: f64,
    pub status: RollbackMilestoneStatus,
}

impl RollbackMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackMilestoneStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackMilestoneStatus) -> bool {
        match self.status {
            RollbackMilestoneStatus::Open => {
                match next {
                    RollbackMilestoneStatus::Resolved | RollbackMilestoneStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackMilestoneStatus::InReview => {
                match next {
                    RollbackMilestoneStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackMilestoneStatus::Resolved => {
                return next == RollbackMilestoneStatus::Reopened || next == RollbackMilestoneStatus::Archived;
            }
            RollbackMilestoneStatus::Reopened => {
                return next == RollbackMilestoneStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.margin + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackMilestone {
        RollbackMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            category: self.category.clone(),
            margin: self.margin,
            cost: self.cost,
            capacity: self.capacity,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a rollback milestone in status open.
pub fn new_rollback_milestone(code: &str, title: &str) -> RollbackMilestone {
    RollbackMilestone {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        category: "".to_string(),
        margin: 0.0,
        cost: 0.0,
        capacity: 0,
        rate: 0.0,
        status: RollbackMilestoneStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rollback_milestone("code-94", "title-138");
        assert_eq!(item.status, RollbackMilestoneStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_rollback_milestone("code-94", "title-138");
        if item.move_to(RollbackMilestoneStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackMilestoneStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rollback_milestone("code-94", "title-138");
        item.status = RollbackMilestoneStatus::Archived;
        assert!(
            item.move_to(RollbackMilestoneStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
