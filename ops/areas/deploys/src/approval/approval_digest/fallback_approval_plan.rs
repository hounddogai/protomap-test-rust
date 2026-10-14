use super::*;

use acme_kit::text;

/// Where a fallback approval plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackApprovalPlanStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackApprovalPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackApprovalPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback approval plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalPlan {
    pub code: String,
    pub title: String,
    pub region: String,
    pub principal: String,
    pub unit: String,
    pub cost: f64,
    pub status: FallbackApprovalPlanStatus,
}

impl FallbackApprovalPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackApprovalPlanStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackApprovalPlanStatus) -> bool {
        match self.status {
            FallbackApprovalPlanStatus::Requested => {
                match next {
                    FallbackApprovalPlanStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackApprovalPlanStatus::Granted => {
                match next {
                    FallbackApprovalPlanStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackApprovalPlanStatus::Revoked => {
                return next == FallbackApprovalPlanStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackApprovalPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback approval plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback approval plan", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> FallbackApprovalPlan {
        FallbackApprovalPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            principal: self.principal.clone(),
            unit: self.unit.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a fallback approval plan in status requested.
pub fn new_fallback_approval_plan(code: &str, title: &str) -> FallbackApprovalPlan {
    FallbackApprovalPlan {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        principal: "".to_string(),
        unit: "".to_string(),
        cost: 0.0,
        status: FallbackApprovalPlanStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_fallback_approval_plan("code-11", "title-142");
        assert_eq!(item.status, FallbackApprovalPlanStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_fallback_approval_plan("code-11", "title-142");
        if item.move_to(FallbackApprovalPlanStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackApprovalPlanStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_fallback_approval_plan("code-11", "title-142");
        item.status = FallbackApprovalPlanStatus::Expired;
        assert!(
            item.move_to(FallbackApprovalPlanStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
