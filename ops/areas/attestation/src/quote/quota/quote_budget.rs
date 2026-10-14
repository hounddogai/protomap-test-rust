use super::*;

use acme_kit::text;

/// Where a quote budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteBudgetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl QuoteBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for QuoteBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote budget as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBudget {
    pub code: String,
    pub title: String,
    pub country: String,
    pub grace_days: i64,
    pub active: bool,
    pub status: QuoteBudgetStatus,
}

impl QuoteBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteBudgetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteBudgetStatus) -> bool {
        match self.status {
            QuoteBudgetStatus::Idle => {
                return next == QuoteBudgetStatus::Warming;
            }
            QuoteBudgetStatus::Warming => {
                return next == QuoteBudgetStatus::Ready;
            }
            QuoteBudgetStatus::Ready => {
                return next == QuoteBudgetStatus::Draining || next == QuoteBudgetStatus::Stopped;
            }
            QuoteBudgetStatus::Draining => {
                match next {
                    QuoteBudgetStatus::Stopped => {
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

    pub fn move_to(&mut self, next: QuoteBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "quote budget", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> QuoteBudget {
        QuoteBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            grace_days: self.grace_days,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a quote budget in status idle.
pub fn new_quote_budget(code: &str, title: &str) -> QuoteBudget {
    QuoteBudget {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        grace_days: 0,
        active: false,
        status: QuoteBudgetStatus::Idle,
    }
}
