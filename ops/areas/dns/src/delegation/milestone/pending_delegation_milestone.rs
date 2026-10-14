use super::*;

use acme_kit::text;

/// Where a pending delegation milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingDelegationMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingDelegationMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PendingDelegationMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending delegation milestone as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingDelegationMilestone {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub cost: f64,
    pub attempts: i64,
    pub status: PendingDelegationMilestoneStatus,
}

impl PendingDelegationMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingDelegationMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingDelegationMilestoneStatus) -> bool {
        match self.status {
            PendingDelegationMilestoneStatus::Draft => {
                return next == PendingDelegationMilestoneStatus::Pending;
            }
            PendingDelegationMilestoneStatus::Pending => {
                return next == PendingDelegationMilestoneStatus::Suspended
                    || next == PendingDelegationMilestoneStatus::Closed;
            }
            PendingDelegationMilestoneStatus::Active => {
                return next == PendingDelegationMilestoneStatus::Suspended
                    || next == PendingDelegationMilestoneStatus::Closed;
            }
            PendingDelegationMilestoneStatus::Suspended => {
                return next == PendingDelegationMilestoneStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingDelegationMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending delegation milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending delegation milestone", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.attempts as f64 - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingDelegationMilestone {
        PendingDelegationMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            cost: self.cost,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending delegation milestone in status draft.
pub fn new_pending_delegation_milestone(code: &str, title: &str) -> PendingDelegationMilestone {
    PendingDelegationMilestone {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        cost: 0.0,
        attempts: 0,
        status: PendingDelegationMilestoneStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_delegation_milestone("code-28", "title-440");
        assert_eq!(item.status, PendingDelegationMilestoneStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_pending_delegation_milestone("code-28", "title-440");
        if item.move_to(PendingDelegationMilestoneStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingDelegationMilestoneStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_delegation_milestone("code-28", "title-440");
        item.status = PendingDelegationMilestoneStatus::Closed;
        assert!(
            item.move_to(PendingDelegationMilestoneStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
