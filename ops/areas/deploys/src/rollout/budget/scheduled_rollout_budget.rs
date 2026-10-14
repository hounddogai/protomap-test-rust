use super::*;

use acme_kit::text;

/// Where a scheduled rollout budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledRolloutBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledRolloutBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ScheduledRolloutBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled rollout budget as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutBudget {
    pub code: String,
    pub label: String,
    pub source: String,
    pub approved: bool,
    pub maximum: f64,
    pub name: String,
    pub batch_size: i64,
    pub country: String,
    pub cost: f64,
    pub scope: String,
    pub namespace: String,
    pub status: ScheduledRolloutBudgetStatus,
}

impl ScheduledRolloutBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledRolloutBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledRolloutBudgetStatus) -> bool {
        match self.status {
            ScheduledRolloutBudgetStatus::Draft => {
                return next == ScheduledRolloutBudgetStatus::Pending;
            }
            ScheduledRolloutBudgetStatus::Pending => {
                match next {
                    ScheduledRolloutBudgetStatus::Active | ScheduledRolloutBudgetStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRolloutBudgetStatus::Active => {
                match next {
                    ScheduledRolloutBudgetStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRolloutBudgetStatus::Suspended => {
                match next {
                    ScheduledRolloutBudgetStatus::Closed => {
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

    pub fn move_to(&mut self, next: ScheduledRolloutBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled rollout budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.maximum - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledRolloutBudget {
        ScheduledRolloutBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            approved: self.approved,
            maximum: self.maximum,
            name: self.name.clone(),
            batch_size: self.batch_size,
            country: self.country.clone(),
            cost: self.cost,
            scope: self.scope.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a scheduled rollout budget in status draft.
pub fn new_scheduled_rollout_budget(code: &str, label: &str) -> ScheduledRolloutBudget {
    ScheduledRolloutBudget {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        approved: false,
        maximum: 0.0,
        name: "".to_string(),
        batch_size: 0,
        country: "".to_string(),
        cost: 0.0,
        scope: "".to_string(),
        namespace: "".to_string(),
        status: ScheduledRolloutBudgetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_rollout_budget("code-680", "label-590");
        assert_eq!(item.status, ScheduledRolloutBudgetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_scheduled_rollout_budget("code-680", "label-590");
        if item.move_to(ScheduledRolloutBudgetStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledRolloutBudgetStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_rollout_budget("code-680", "label-590");
        item.status = ScheduledRolloutBudgetStatus::Closed;
        assert!(
            item.move_to(ScheduledRolloutBudgetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
