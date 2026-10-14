use super::*;

use acme_kit::text;

/// Where a usage budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageBudgetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl UsageBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for UsageBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One usage budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageBudget {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub scope: String,
    pub revision: i64,
    pub audience: String,
    pub cluster: String,
    pub duration_minutes: i64,
    pub status: UsageBudgetStatus,
}

impl UsageBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageBudgetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageBudgetStatus) -> bool {
        match self.status {
            UsageBudgetStatus::Open => {
                match next {
                    UsageBudgetStatus::Reopened | UsageBudgetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageBudgetStatus::InReview => {
                match next {
                    UsageBudgetStatus::Reopened | UsageBudgetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageBudgetStatus::Resolved => {
                match next {
                    UsageBudgetStatus::Reopened | UsageBudgetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageBudgetStatus::Reopened => {
                return next == UsageBudgetStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.revision as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageBudget {
        UsageBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            scope: self.scope.clone(),
            revision: self.revision,
            audience: self.audience.clone(),
            cluster: self.cluster.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a usage budget in status open.
pub fn new_usage_budget(code: &str, label: &str) -> UsageBudget {
    UsageBudget {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        scope: "".to_string(),
        revision: 0,
        audience: "".to_string(),
        cluster: "".to_string(),
        duration_minutes: 0,
        status: UsageBudgetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_usage_budget("code-44", "label-261");
        assert_eq!(item.status, UsageBudgetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_usage_budget("code-44", "label-261");
        if item.move_to(UsageBudgetStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageBudgetStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_usage_budget("code-44", "label-261");
        item.status = UsageBudgetStatus::Archived;
        assert!(item.move_to(UsageBudgetStatus::Open).is_err(), "expected move to fail");
    }
}
