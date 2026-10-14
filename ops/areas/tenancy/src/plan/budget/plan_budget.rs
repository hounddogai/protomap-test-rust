use super::*;

use acme_kit::text;

/// Where a plan budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanBudgetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PlanBudgetStatus {
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

impl std::fmt::Display for PlanBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One plan budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanBudget {
    pub code: String,
    pub name: String,
    pub country: String,
    pub amount: f64,
    pub summary: String,
    pub ceiling: f64,
    pub checksum: String,
    pub notes: String,
    pub floor_value: f64,
    pub margin: f64,
    pub title: String,
    pub status: PlanBudgetStatus,
}

impl PlanBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanBudgetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanBudgetStatus) -> bool {
        match self.status {
            PlanBudgetStatus::Planned => {
                return next == PlanBudgetStatus::Completed || next == PlanBudgetStatus::Cancelled;
            }
            PlanBudgetStatus::Scheduled => {
                return next == PlanBudgetStatus::Completed;
            }
            PlanBudgetStatus::InProgress => {
                return next == PlanBudgetStatus::Completed;
            }
            PlanBudgetStatus::Completed => {
                match next {
                    PlanBudgetStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PlanBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.margin - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanBudget {
        PlanBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            amount: self.amount,
            summary: self.summary.clone(),
            ceiling: self.ceiling,
            checksum: self.checksum.clone(),
            notes: self.notes.clone(),
            floor_value: self.floor_value,
            margin: self.margin,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a plan budget in status planned.
pub fn new_plan_budget(code: &str, name: &str) -> PlanBudget {
    PlanBudget {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        amount: 0.0,
        summary: "".to_string(),
        ceiling: 0.0,
        checksum: "".to_string(),
        notes: "".to_string(),
        floor_value: 0.0,
        margin: 0.0,
        title: "".to_string(),
        status: PlanBudgetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_plan_budget("code-699", "name-363");
        assert_eq!(item.status, PlanBudgetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_plan_budget("code-699", "name-363");
        if item.move_to(PlanBudgetStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanBudgetStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_plan_budget("code-699", "name-363");
        item.status = PlanBudgetStatus::Cancelled;
        assert!(
            item.move_to(PlanBudgetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
