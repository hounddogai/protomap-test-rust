use super::*;

use acme_kit::text;

/// Where a legacy silence budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacySilenceBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacySilenceBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacySilenceBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy silence budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceBudget {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub rate: f64,
    pub label: String,
    pub checksum: String,
    pub amount: f64,
    pub status: LegacySilenceBudgetStatus,
}

impl LegacySilenceBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacySilenceBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacySilenceBudgetStatus) -> bool {
        match self.status {
            LegacySilenceBudgetStatus::Created => {
                return next == LegacySilenceBudgetStatus::Validated || next == LegacySilenceBudgetStatus::Published;
            }
            LegacySilenceBudgetStatus::Validated => {
                return next == LegacySilenceBudgetStatus::Retired;
            }
            LegacySilenceBudgetStatus::Published => {
                return next == LegacySilenceBudgetStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacySilenceBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy silence budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        (self.rate * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacySilenceBudget {
        LegacySilenceBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            rate: self.rate,
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy silence budget in status created.
pub fn new_legacy_silence_budget(code: &str, title: &str) -> LegacySilenceBudget {
    LegacySilenceBudget {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        rate: 0.0,
        label: "".to_string(),
        checksum: "".to_string(),
        amount: 0.0,
        status: LegacySilenceBudgetStatus::Created,
    }
}
