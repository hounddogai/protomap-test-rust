use super::*;

use acme_kit::text;

/// Where a scheduled query budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledQueryBudgetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledQueryBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledQueryBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled query budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryBudget {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub maximum: f64,
    pub margin: f64,
    pub expires_at: i64,
    pub created_at: i64,
    pub tenant_id: String,
    pub window_days: i64,
    pub status: ScheduledQueryBudgetStatus,
}

impl ScheduledQueryBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledQueryBudgetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledQueryBudgetStatus) -> bool {
        match self.status {
            ScheduledQueryBudgetStatus::Proposed => {
                return next == ScheduledQueryBudgetStatus::Withdrawn;
            }
            ScheduledQueryBudgetStatus::Approved => {
                match next {
                    ScheduledQueryBudgetStatus::Rejected | ScheduledQueryBudgetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledQueryBudgetStatus::Rejected => {
                return next == ScheduledQueryBudgetStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledQueryBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled query budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.created_at as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledQueryBudget {
        ScheduledQueryBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            maximum: self.maximum,
            margin: self.margin,
            expires_at: self.expires_at,
            created_at: self.created_at,
            tenant_id: self.tenant_id.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled query budget in status proposed.
pub fn new_scheduled_query_budget(code: &str, label: &str) -> ScheduledQueryBudget {
    ScheduledQueryBudget {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        maximum: 0.0,
        margin: 0.0,
        expires_at: 0,
        created_at: 0,
        tenant_id: "".to_string(),
        window_days: 0,
        status: ScheduledQueryBudgetStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_scheduled_query_budget("code-943", "label-614");
        assert_eq!(item.status, ScheduledQueryBudgetStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_scheduled_query_budget("code-943", "label-614");
        if item.move_to(ScheduledQueryBudgetStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledQueryBudgetStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_scheduled_query_budget("code-943", "label-614");
        item.status = ScheduledQueryBudgetStatus::Withdrawn;
        assert!(
            item.move_to(ScheduledQueryBudgetStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
