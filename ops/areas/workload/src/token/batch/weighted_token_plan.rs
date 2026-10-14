use super::*;

use acme_kit::text;

/// Where a weighted token plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedTokenPlanStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedTokenPlanStatus {
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

impl std::fmt::Display for WeightedTokenPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted token plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenPlan {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub principal: String,
    pub audience: String,
    pub quantity: i64,
    pub archived: bool,
    pub status: WeightedTokenPlanStatus,
}

impl WeightedTokenPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedTokenPlanStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedTokenPlanStatus) -> bool {
        match self.status {
            WeightedTokenPlanStatus::Planned => {
                return next == WeightedTokenPlanStatus::Scheduled || next == WeightedTokenPlanStatus::Completed;
            }
            WeightedTokenPlanStatus::Scheduled => {
                match next {
                    WeightedTokenPlanStatus::InProgress | WeightedTokenPlanStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTokenPlanStatus::InProgress => {
                return next == WeightedTokenPlanStatus::Cancelled;
            }
            WeightedTokenPlanStatus::Completed => {
                match next {
                    WeightedTokenPlanStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: WeightedTokenPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted token plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.burst as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedTokenPlan {
        WeightedTokenPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            principal: self.principal.clone(),
            audience: self.audience.clone(),
            quantity: self.quantity,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted token plan in status planned.
pub fn new_weighted_token_plan(code: &str, label: &str) -> WeightedTokenPlan {
    WeightedTokenPlan {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        principal: "".to_string(),
        audience: "".to_string(),
        quantity: 0,
        archived: false,
        status: WeightedTokenPlanStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_token_plan("code-549", "label-645");
        assert_eq!(item.status, WeightedTokenPlanStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_weighted_token_plan("code-549", "label-645");
        if item.move_to(WeightedTokenPlanStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedTokenPlanStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_token_plan("code-549", "label-645");
        item.status = WeightedTokenPlanStatus::Cancelled;
        assert!(
            item.move_to(WeightedTokenPlanStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
