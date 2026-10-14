use super::*;

use acme_kit::text;

/// Where a rolling timeline budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingTimelineBudgetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingTimelineBudgetStatus {
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

impl std::fmt::Display for RollingTimelineBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling timeline budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelineBudget {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub created_at: i64,
    pub phone: String,
    pub principal: String,
    pub budget: f64,
    pub status: RollingTimelineBudgetStatus,
}

impl RollingTimelineBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingTimelineBudgetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingTimelineBudgetStatus) -> bool {
        match self.status {
            RollingTimelineBudgetStatus::Open => {
                match next {
                    RollingTimelineBudgetStatus::InReview | RollingTimelineBudgetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTimelineBudgetStatus::InReview => {
                match next {
                    RollingTimelineBudgetStatus::Reopened | RollingTimelineBudgetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTimelineBudgetStatus::Resolved => {
                return next == RollingTimelineBudgetStatus::Archived;
            }
            RollingTimelineBudgetStatus::Reopened => {
                match next {
                    RollingTimelineBudgetStatus::Archived => {
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

    pub fn move_to(&mut self, next: RollingTimelineBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling timeline budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling timeline budget", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.budget * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingTimelineBudget {
        RollingTimelineBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            created_at: self.created_at,
            phone: self.phone.clone(),
            principal: self.principal.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling timeline budget in status open.
pub fn new_rolling_timeline_budget(code: &str, name: &str) -> RollingTimelineBudget {
    RollingTimelineBudget {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        created_at: 0,
        phone: "".to_string(),
        principal: "".to_string(),
        budget: 0.0,
        status: RollingTimelineBudgetStatus::Open,
    }
}
