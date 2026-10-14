use super::*;

use acme_kit::text;

/// Where a budget burn slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnSlotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BudgetBurnSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BudgetBurnSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn slot as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnSlot {
    pub code: String,
    pub title: String,
    pub country: String,
    pub rotation_days: i64,
    pub label: String,
    pub host: String,
    pub region: String,
    pub batch_size: i64,
    pub locale: String,
    pub status: BudgetBurnSlotStatus,
}

impl BudgetBurnSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnSlotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnSlotStatus) -> bool {
        match self.status {
            BudgetBurnSlotStatus::Proposed => {
                return next == BudgetBurnSlotStatus::Approved;
            }
            BudgetBurnSlotStatus::Approved => {
                match next {
                    BudgetBurnSlotStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnSlotStatus::Rejected => {
                return next == BudgetBurnSlotStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.batch_size as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnSlot {
        BudgetBurnSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            rotation_days: self.rotation_days,
            label: self.label.clone(),
            host: self.host.clone(),
            region: self.region.clone(),
            batch_size: self.batch_size,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a budget burn slot in status proposed.
pub fn new_budget_burn_slot(code: &str, title: &str) -> BudgetBurnSlot {
    BudgetBurnSlot {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        rotation_days: 0,
        label: "".to_string(),
        host: "".to_string(),
        region: "".to_string(),
        batch_size: 0,
        locale: "".to_string(),
        status: BudgetBurnSlotStatus::Proposed,
    }
}
