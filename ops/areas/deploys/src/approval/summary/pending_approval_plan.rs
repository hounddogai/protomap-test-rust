use super::*;

use acme_kit::text;

/// Where a pending approval plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingApprovalPlanStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingApprovalPlanStatus {
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

impl std::fmt::Display for PendingApprovalPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending approval plan as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalPlan {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub country: String,
    pub checksum: String,
    pub ceiling: f64,
    pub cost: f64,
    pub status: PendingApprovalPlanStatus,
}

impl PendingApprovalPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingApprovalPlanStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingApprovalPlanStatus) -> bool {
        match self.status {
            PendingApprovalPlanStatus::Draft => {
                return next == PendingApprovalPlanStatus::Active || next == PendingApprovalPlanStatus::Suspended;
            }
            PendingApprovalPlanStatus::Pending => {
                return next == PendingApprovalPlanStatus::Closed;
            }
            PendingApprovalPlanStatus::Active => {
                return next == PendingApprovalPlanStatus::Suspended || next == PendingApprovalPlanStatus::Closed;
            }
            PendingApprovalPlanStatus::Suspended => {
                return next == PendingApprovalPlanStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingApprovalPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending approval plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.ceiling * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingApprovalPlan {
        PendingApprovalPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            country: self.country.clone(),
            checksum: self.checksum.clone(),
            ceiling: self.ceiling,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a pending approval plan in status draft.
pub fn new_pending_approval_plan(code: &str, name: &str) -> PendingApprovalPlan {
    PendingApprovalPlan {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        country: "".to_string(),
        checksum: "".to_string(),
        ceiling: 0.0,
        cost: 0.0,
        status: PendingApprovalPlanStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_approval_plan("code-718", "name-676");
        assert_eq!(item.status, PendingApprovalPlanStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_pending_approval_plan("code-718", "name-676");
        if item.move_to(PendingApprovalPlanStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingApprovalPlanStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_approval_plan("code-718", "name-676");
        item.status = PendingApprovalPlanStatus::Closed;
        assert!(
            item.move_to(PendingApprovalPlanStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
