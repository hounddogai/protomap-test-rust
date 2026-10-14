use super::*;

use acme_kit::text;

/// Where a tiered budget burn slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredBudgetBurnSlotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TieredBudgetBurnSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TieredBudgetBurnSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered budget burn slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnSlot {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub ceiling: f64,
    pub summary: String,
    pub host: String,
    pub rotation_days: i64,
    pub window_days: i64,
    pub status: TieredBudgetBurnSlotStatus,
}

impl TieredBudgetBurnSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredBudgetBurnSlotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredBudgetBurnSlotStatus) -> bool {
        match self.status {
            TieredBudgetBurnSlotStatus::Requested => {
                return next == TieredBudgetBurnSlotStatus::Revoked || next == TieredBudgetBurnSlotStatus::Expired;
            }
            TieredBudgetBurnSlotStatus::Granted => {
                return next == TieredBudgetBurnSlotStatus::Revoked || next == TieredBudgetBurnSlotStatus::Expired;
            }
            TieredBudgetBurnSlotStatus::Revoked => {
                return next == TieredBudgetBurnSlotStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredBudgetBurnSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered budget burn slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered budget burn slot", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.burst as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredBudgetBurnSlot {
        TieredBudgetBurnSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            ceiling: self.ceiling,
            summary: self.summary.clone(),
            host: self.host.clone(),
            rotation_days: self.rotation_days,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered budget burn slot in status requested.
pub fn new_tiered_budget_burn_slot(code: &str, label: &str) -> TieredBudgetBurnSlot {
    TieredBudgetBurnSlot {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        ceiling: 0.0,
        summary: "".to_string(),
        host: "".to_string(),
        rotation_days: 0,
        window_days: 0,
        status: TieredBudgetBurnSlotStatus::Requested,
    }
}
