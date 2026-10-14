use super::*;

use acme_kit::text;

/// Where a budget burn reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnReservationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BudgetBurnReservationStatus {
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

impl std::fmt::Display for BudgetBurnReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnReservation {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub ratio: f64,
    pub limit: i64,
    pub burst: i64,
    pub title: String,
    pub attempts: i64,
    pub budget: f64,
    pub minimum: f64,
    pub status: BudgetBurnReservationStatus,
}

impl BudgetBurnReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnReservationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnReservationStatus) -> bool {
        match self.status {
            BudgetBurnReservationStatus::Planned => {
                return next == BudgetBurnReservationStatus::Completed;
            }
            BudgetBurnReservationStatus::Scheduled => {
                return next == BudgetBurnReservationStatus::Completed;
            }
            BudgetBurnReservationStatus::InProgress => {
                return next == BudgetBurnReservationStatus::Completed
                    || next == BudgetBurnReservationStatus::Cancelled;
            }
            BudgetBurnReservationStatus::Completed => {
                return next == BudgetBurnReservationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "budget burn reservation", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.ratio - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnReservation {
        BudgetBurnReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            ratio: self.ratio,
            limit: self.limit,
            burst: self.burst,
            title: self.title.clone(),
            attempts: self.attempts,
            budget: self.budget,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a budget burn reservation in status planned.
pub fn new_budget_burn_reservation(code: &str, name: &str) -> BudgetBurnReservation {
    BudgetBurnReservation {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        ratio: 0.0,
        limit: 0,
        burst: 0,
        title: "".to_string(),
        attempts: 0,
        budget: 0.0,
        minimum: 0.0,
        status: BudgetBurnReservationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_budget_burn_reservation("code-739", "name-571");
        assert_eq!(item.status, BudgetBurnReservationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_budget_burn_reservation("code-739", "name-571");
        if item.move_to(BudgetBurnReservationStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnReservationStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_budget_burn_reservation("code-739", "name-571");
        item.status = BudgetBurnReservationStatus::Cancelled;
        assert!(
            item.move_to(BudgetBurnReservationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
