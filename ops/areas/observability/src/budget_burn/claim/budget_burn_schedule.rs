use super::*;

use acme_kit::text;

/// Where a budget burn schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnScheduleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BudgetBurnScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BudgetBurnScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnSchedule {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub scope: String,
    pub title: String,
    pub rate: f64,
    pub category: String,
    pub currency: String,
    pub revision: i64,
    pub ttl_seconds: i64,
    pub status: BudgetBurnScheduleStatus,
}

impl BudgetBurnSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnScheduleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnScheduleStatus) -> bool {
        match self.status {
            BudgetBurnScheduleStatus::Requested => {
                return next == BudgetBurnScheduleStatus::Granted;
            }
            BudgetBurnScheduleStatus::Granted => {
                match next {
                    BudgetBurnScheduleStatus::Revoked | BudgetBurnScheduleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnScheduleStatus::Revoked => {
                return next == BudgetBurnScheduleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.revision as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnSchedule {
        BudgetBurnSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            scope: self.scope.clone(),
            title: self.title.clone(),
            rate: self.rate,
            category: self.category.clone(),
            currency: self.currency.clone(),
            revision: self.revision,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a budget burn schedule in status requested.
pub fn new_budget_burn_schedule(code: &str, name: &str) -> BudgetBurnSchedule {
    BudgetBurnSchedule {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        scope: "".to_string(),
        title: "".to_string(),
        rate: 0.0,
        category: "".to_string(),
        currency: "".to_string(),
        revision: 0,
        ttl_seconds: 0,
        status: BudgetBurnScheduleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_budget_burn_schedule("code-921", "name-39");
        assert_eq!(item.status, BudgetBurnScheduleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_budget_burn_schedule("code-921", "name-39");
        if item.move_to(BudgetBurnScheduleStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnScheduleStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_budget_burn_schedule("code-921", "name-39");
        item.status = BudgetBurnScheduleStatus::Expired;
        assert!(
            item.move_to(BudgetBurnScheduleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
