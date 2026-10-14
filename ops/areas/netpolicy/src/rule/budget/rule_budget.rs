use super::*;

use acme_kit::text;

/// Where a rule budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleBudgetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RuleBudgetStatus {
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

impl std::fmt::Display for RuleBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rule budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleBudget {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub score: f64,
    pub channel: String,
    pub external_ref: String,
    pub approved: bool,
    pub reviewer: String,
    pub floor_value: f64,
    pub priority: i64,
    pub status: RuleBudgetStatus,
}

impl RuleBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleBudgetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleBudgetStatus) -> bool {
        match self.status {
            RuleBudgetStatus::Planned => {
                return next == RuleBudgetStatus::Scheduled;
            }
            RuleBudgetStatus::Scheduled => {
                return next == RuleBudgetStatus::InProgress;
            }
            RuleBudgetStatus::InProgress => {
                match next {
                    RuleBudgetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleBudgetStatus::Completed => {
                return next == RuleBudgetStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.priority as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleBudget {
        RuleBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            score: self.score,
            channel: self.channel.clone(),
            external_ref: self.external_ref.clone(),
            approved: self.approved,
            reviewer: self.reviewer.clone(),
            floor_value: self.floor_value,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a rule budget in status planned.
pub fn new_rule_budget(code: &str, label: &str) -> RuleBudget {
    RuleBudget {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        score: 0.0,
        channel: "".to_string(),
        external_ref: "".to_string(),
        approved: false,
        reviewer: "".to_string(),
        floor_value: 0.0,
        priority: 0,
        status: RuleBudgetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rule_budget("code-745", "label-367");
        assert_eq!(item.status, RuleBudgetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_rule_budget("code-745", "label-367");
        if item.move_to(RuleBudgetStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleBudgetStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rule_budget("code-745", "label-367");
        item.status = RuleBudgetStatus::Cancelled;
        assert!(
            item.move_to(RuleBudgetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
