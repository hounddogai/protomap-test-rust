use super::*;

use acme_kit::text;

/// Where a freeze budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FreezeBudgetStatus {
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

impl std::fmt::Display for FreezeBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a freeze budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBudget {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub approved: bool,
    pub threshold: f64,
    pub status: FreezeBudgetStatus,
}

impl FreezeBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeBudgetStatus) -> bool {
        match self.status {
            FreezeBudgetStatus::Draft => {
                match next {
                    FreezeBudgetStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeBudgetStatus::Pending => {
                return next == FreezeBudgetStatus::Suspended || next == FreezeBudgetStatus::Closed;
            }
            FreezeBudgetStatus::Active => {
                match next {
                    FreezeBudgetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeBudgetStatus::Suspended => {
                return next == FreezeBudgetStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.threshold - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeBudget {
        FreezeBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            approved: self.approved,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a freeze budget in status draft.
pub fn new_freeze_budget(code: &str, title: &str) -> FreezeBudget {
    FreezeBudget {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        approved: false,
        threshold: 0.0,
        status: FreezeBudgetStatus::Draft,
    }
}
