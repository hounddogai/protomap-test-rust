use super::*;

use acme_kit::text;

/// Where a strict budget burn window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictBudgetBurnWindowStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictBudgetBurnWindowStatus {
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

impl std::fmt::Display for StrictBudgetBurnWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict budget burn window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnWindow {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub budget: f64,
    pub notes: String,
    pub minimum: f64,
    pub source: String,
    pub amount: f64,
    pub active: bool,
    pub ceiling: f64,
    pub status: StrictBudgetBurnWindowStatus,
}

impl StrictBudgetBurnWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictBudgetBurnWindowStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictBudgetBurnWindowStatus) -> bool {
        match self.status {
            StrictBudgetBurnWindowStatus::Draft => {
                return next == StrictBudgetBurnWindowStatus::Pending;
            }
            StrictBudgetBurnWindowStatus::Pending => {
                return next == StrictBudgetBurnWindowStatus::Active;
            }
            StrictBudgetBurnWindowStatus::Active => {
                match next {
                    StrictBudgetBurnWindowStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictBudgetBurnWindowStatus::Suspended => {
                return next == StrictBudgetBurnWindowStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictBudgetBurnWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict budget burn window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.ceiling + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictBudgetBurnWindow {
        StrictBudgetBurnWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            budget: self.budget,
            notes: self.notes.clone(),
            minimum: self.minimum,
            source: self.source.clone(),
            amount: self.amount,
            active: self.active,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a strict budget burn window in status draft.
pub fn new_strict_budget_burn_window(code: &str, title: &str) -> StrictBudgetBurnWindow {
    StrictBudgetBurnWindow {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        budget: 0.0,
        notes: "".to_string(),
        minimum: 0.0,
        source: "".to_string(),
        amount: 0.0,
        active: false,
        ceiling: 0.0,
        status: StrictBudgetBurnWindowStatus::Draft,
    }
}
