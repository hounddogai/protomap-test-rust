use super::*;

use acme_kit::text;

/// Where a capped budget burn schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedBudgetBurnScheduleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedBudgetBurnScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedBudgetBurnScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped budget burn schedule as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnSchedule {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub audience: String,
    pub segment: String,
    pub grace_days: i64,
    pub title: String,
    pub locale: String,
    pub status: CappedBudgetBurnScheduleStatus,
}

impl CappedBudgetBurnSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedBudgetBurnScheduleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedBudgetBurnScheduleStatus) -> bool {
        match self.status {
            CappedBudgetBurnScheduleStatus::Requested => {
                match next {
                    CappedBudgetBurnScheduleStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedBudgetBurnScheduleStatus::Granted => {
                return next == CappedBudgetBurnScheduleStatus::Revoked
                    || next == CappedBudgetBurnScheduleStatus::Expired;
            }
            CappedBudgetBurnScheduleStatus::Revoked => {
                return next == CappedBudgetBurnScheduleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedBudgetBurnScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped budget burn schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedBudgetBurnSchedule {
        CappedBudgetBurnSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            audience: self.audience.clone(),
            segment: self.segment.clone(),
            grace_days: self.grace_days,
            title: self.title.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a capped budget burn schedule in status requested.
pub fn new_capped_budget_burn_schedule(code: &str, label: &str) -> CappedBudgetBurnSchedule {
    CappedBudgetBurnSchedule {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        audience: "".to_string(),
        segment: "".to_string(),
        grace_days: 0,
        title: "".to_string(),
        locale: "".to_string(),
        status: CappedBudgetBurnScheduleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_capped_budget_burn_schedule("code-728", "label-321");
        assert_eq!(item.status, CappedBudgetBurnScheduleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_capped_budget_burn_schedule("code-728", "label-321");
        if item.move_to(CappedBudgetBurnScheduleStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedBudgetBurnScheduleStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_capped_budget_burn_schedule("code-728", "label-321");
        item.status = CappedBudgetBurnScheduleStatus::Expired;
        assert!(
            item.move_to(CappedBudgetBurnScheduleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
