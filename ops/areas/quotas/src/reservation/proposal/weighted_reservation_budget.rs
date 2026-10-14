use super::*;

use acme_kit::text;

/// Where a weighted reservation budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedReservationBudgetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedReservationBudgetStatus {
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

impl std::fmt::Display for WeightedReservationBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted reservation budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedReservationBudget {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub weight: f64,
    pub active: bool,
    pub grace_days: i64,
    pub minimum: f64,
    pub title: String,
    pub fingerprint: String,
    pub issuer: String,
    pub ceiling: f64,
    pub status: WeightedReservationBudgetStatus,
}

impl WeightedReservationBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedReservationBudgetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedReservationBudgetStatus) -> bool {
        match self.status {
            WeightedReservationBudgetStatus::Planned => {
                return next == WeightedReservationBudgetStatus::Completed;
            }
            WeightedReservationBudgetStatus::Scheduled => {
                match next {
                    WeightedReservationBudgetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedReservationBudgetStatus::InProgress => {
                match next {
                    WeightedReservationBudgetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedReservationBudgetStatus::Completed => {
                return next == WeightedReservationBudgetStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedReservationBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted reservation budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.weight - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedReservationBudget {
        WeightedReservationBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            weight: self.weight,
            active: self.active,
            grace_days: self.grace_days,
            minimum: self.minimum,
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            issuer: self.issuer.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a weighted reservation budget in status planned.
pub fn new_weighted_reservation_budget(code: &str, name: &str) -> WeightedReservationBudget {
    WeightedReservationBudget {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        weight: 0.0,
        active: false,
        grace_days: 0,
        minimum: 0.0,
        title: "".to_string(),
        fingerprint: "".to_string(),
        issuer: "".to_string(),
        ceiling: 0.0,
        status: WeightedReservationBudgetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_reservation_budget("code-637", "name-145");
        assert_eq!(item.status, WeightedReservationBudgetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_weighted_reservation_budget("code-637", "name-145");
        if item.move_to(WeightedReservationBudgetStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedReservationBudgetStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_reservation_budget("code-637", "name-145");
        item.status = WeightedReservationBudgetStatus::Cancelled;
        assert!(
            item.move_to(WeightedReservationBudgetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
