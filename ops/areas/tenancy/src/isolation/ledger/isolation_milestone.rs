use super::*;

use acme_kit::text;

/// Where a isolation milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationMilestoneStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl IsolationMilestoneStatus {
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

impl std::fmt::Display for IsolationMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation milestone as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationMilestone {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub revision: i64,
    pub region: String,
    pub grace_days: i64,
    pub tags: Vec<String>,
    pub amount: f64,
    pub status: IsolationMilestoneStatus,
}

impl IsolationMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationMilestoneStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationMilestoneStatus) -> bool {
        match self.status {
            IsolationMilestoneStatus::Open => {
                return next == IsolationMilestoneStatus::Resolved;
            }
            IsolationMilestoneStatus::InReview => {
                return next == IsolationMilestoneStatus::Resolved || next == IsolationMilestoneStatus::Reopened;
            }
            IsolationMilestoneStatus::Resolved => {
                match next {
                    IsolationMilestoneStatus::Reopened | IsolationMilestoneStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationMilestoneStatus::Reopened => {
                return next == IsolationMilestoneStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.amount * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationMilestone {
        IsolationMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            revision: self.revision,
            region: self.region.clone(),
            grace_days: self.grace_days,
            tags: self.tags.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a isolation milestone in status open.
pub fn new_isolation_milestone(code: &str, label: &str) -> IsolationMilestone {
    IsolationMilestone {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        revision: 0,
        region: "".to_string(),
        grace_days: 0,
        tags: Vec::new(),
        amount: 0.0,
        status: IsolationMilestoneStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_isolation_milestone("code-855", "label-8");
        assert_eq!(item.status, IsolationMilestoneStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_isolation_milestone("code-855", "label-8");
        if item.move_to(IsolationMilestoneStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationMilestoneStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_isolation_milestone("code-855", "label-8");
        item.status = IsolationMilestoneStatus::Archived;
        assert!(
            item.move_to(IsolationMilestoneStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
