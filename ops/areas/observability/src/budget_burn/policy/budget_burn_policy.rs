use super::*;

use acme_kit::text;

/// Where a budget burn policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnPolicyStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BudgetBurnPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for BudgetBurnPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One budget burn policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPolicy {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub batch_size: i64,
    pub quota_units: i64,
    pub status: BudgetBurnPolicyStatus,
}

impl BudgetBurnPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnPolicyStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnPolicyStatus) -> bool {
        match self.status {
            BudgetBurnPolicyStatus::Planned => {
                match next {
                    BudgetBurnPolicyStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnPolicyStatus::Scheduled => {
                match next {
                    BudgetBurnPolicyStatus::Completed | BudgetBurnPolicyStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnPolicyStatus::InProgress => {
                match next {
                    BudgetBurnPolicyStatus::Completed | BudgetBurnPolicyStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnPolicyStatus::Completed => {
                return next == BudgetBurnPolicyStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.quota_units as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnPolicy {
        BudgetBurnPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            batch_size: self.batch_size,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a budget burn policy in status planned.
pub fn new_budget_burn_policy(code: &str, name: &str) -> BudgetBurnPolicy {
    BudgetBurnPolicy {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        batch_size: 0,
        quota_units: 0,
        status: BudgetBurnPolicyStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_budget_burn_policy("code-803", "name-502");
        assert_eq!(item.status, BudgetBurnPolicyStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_budget_burn_policy("code-803", "name-502");
        if item.move_to(BudgetBurnPolicyStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnPolicyStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_budget_burn_policy("code-803", "name-502");
        item.status = BudgetBurnPolicyStatus::Cancelled;
        assert!(
            item.move_to(BudgetBurnPolicyStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
