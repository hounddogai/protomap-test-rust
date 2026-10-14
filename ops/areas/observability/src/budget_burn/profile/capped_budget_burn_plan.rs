use super::*;

use acme_kit::text;

/// Where a capped budget burn plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedBudgetBurnPlanStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CappedBudgetBurnPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for CappedBudgetBurnPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped budget burn plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnPlan {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub updated_at: i64,
    pub ttl_seconds: i64,
    pub status: CappedBudgetBurnPlanStatus,
}

impl CappedBudgetBurnPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedBudgetBurnPlanStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedBudgetBurnPlanStatus) -> bool {
        match self.status {
            CappedBudgetBurnPlanStatus::Idle => {
                match next {
                    CappedBudgetBurnPlanStatus::Warming | CappedBudgetBurnPlanStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedBudgetBurnPlanStatus::Warming => {
                return next == CappedBudgetBurnPlanStatus::Draining;
            }
            CappedBudgetBurnPlanStatus::Ready => {
                match next {
                    CappedBudgetBurnPlanStatus::Draining | CappedBudgetBurnPlanStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedBudgetBurnPlanStatus::Draining => {
                match next {
                    CappedBudgetBurnPlanStatus::Stopped => {
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

    pub fn move_to(&mut self, next: CappedBudgetBurnPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped budget burn plan {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedBudgetBurnPlan {
        CappedBudgetBurnPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            updated_at: self.updated_at,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped budget burn plan in status idle.
pub fn new_capped_budget_burn_plan(code: &str, label: &str) -> CappedBudgetBurnPlan {
    CappedBudgetBurnPlan {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        updated_at: 0,
        ttl_seconds: 0,
        status: CappedBudgetBurnPlanStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_capped_budget_burn_plan("code-848", "label-721");
        assert_eq!(item.status, CappedBudgetBurnPlanStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_capped_budget_burn_plan("code-848", "label-721");
        if item.move_to(CappedBudgetBurnPlanStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedBudgetBurnPlanStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_capped_budget_burn_plan("code-848", "label-721");
        item.status = CappedBudgetBurnPlanStatus::Stopped;
        assert!(
            item.move_to(CappedBudgetBurnPlanStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
